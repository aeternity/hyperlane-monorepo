use async_trait::async_trait;

use hyperlane_core::{
    accumulator::incremental::IncrementalMerkle, ChainResult, Checkpoint, CheckpointAtBlock,
    ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain, HyperlaneProvider,
    IncrementalMerkleAtBlock, MerkleTreeHook, ReorgPeriod, H256,
};

use crate::{
    contracts, h256_to_contract_address, AeternityProvider, HyperlaneAeternityError,
};

/// Number of branch nodes in the incremental Merkle tree (depth = 32).
const TREE_DEPTH: usize = 32;

/// Aeternity Merkle Tree Hook
#[derive(Debug)]
pub struct AeMerkleTreeHook {
    provider: AeternityProvider,
    contract_address: String,
    address_h256: H256,
    domain: HyperlaneDomain,
}

impl AeMerkleTreeHook {
    /// Creates a new Aeternity MerkleTreeHook instance
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address);
        Ok(Self {
            domain: provider.domain().clone(),
            provider,
            contract_address,
            address_h256: locator.address,
        })
    }

    /// Parse the compiler-decoded tree value into an `IncrementalMerkle`.
    ///
    /// The compiler decodes `{ branch: map(int, bytes(32)), count: int }` as a
    /// JSON object with `"branch"` and `"count"` fields.
    fn parse_tree(value: &serde_json::Value) -> ChainResult<IncrementalMerkle> {
        let obj = value.as_object().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected object from tree(), got {value}"
            ))
        })?;

        let count = obj
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let mut branch = [H256::zero(); TREE_DEPTH];
        if let Some(branch_map) = obj.get("branch").and_then(|v| v.as_object()) {
            for (key, val) in branch_map {
                if let (Ok(idx), Some(hex_str)) = (key.parse::<usize>(), val.as_str()) {
                    if idx < TREE_DEPTH {
                        let hex_clean = hex_str.trim_start_matches('#');
                        if let Ok(bytes) = hex::decode(hex_clean) {
                            if bytes.len() == 32 {
                                branch[idx] = H256::from_slice(&bytes);
                            }
                        }
                    }
                }
            }
        }

        Ok(IncrementalMerkle::new(branch, count))
    }

    /// Parse the compiler-decoded checkpoint value into `(root, index)`.
    ///
    /// The compiler decodes `bytes(32) * int` as a JSON array `[hex_bytes, int]`.
    fn parse_checkpoint(value: &serde_json::Value) -> ChainResult<(H256, u32)> {
        let arr = value.as_array().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected tuple array from latest_checkpoint(), got {value}"
            ))
        })?;

        if arr.len() != 2 {
            return Err(HyperlaneAeternityError::ContractCallError(format!(
                "expected 2-element tuple from latest_checkpoint(), got {} elements",
                arr.len()
            ))
            .into());
        }

        let root = parse_bytes32(&arr[0])?;
        let index = arr[1].as_u64().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected integer for checkpoint index, got {}",
                arr[1]
            ))
        })? as u32;

        Ok((root, index))
    }
}

/// Parse a compiler-decoded `bytes(32)` value (hex string with `#` prefix).
fn parse_bytes32(value: &serde_json::Value) -> ChainResult<H256> {
    let hex_str = value.as_str().ok_or_else(|| {
        HyperlaneAeternityError::ContractCallError(format!(
            "expected hex string for bytes(32), got {value}"
        ))
    })?;
    let hex_clean = hex_str.trim_start_matches('#');
    let bytes = hex::decode(hex_clean).map_err(|e| {
        HyperlaneAeternityError::ContractCallError(format!("invalid hex for bytes(32): {e}"))
    })?;
    if bytes.len() != 32 {
        return Err(HyperlaneAeternityError::ContractCallError(format!(
            "expected 32 bytes, got {}",
            bytes.len()
        ))
        .into());
    }
    Ok(H256::from_slice(&bytes))
}

impl HyperlaneContract for AeMerkleTreeHook {
    fn address(&self) -> H256 {
        self.address_h256
    }
}

impl HyperlaneChain for AeMerkleTreeHook {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(self.provider.clone())
    }
}

#[async_trait]
impl MerkleTreeHook for AeMerkleTreeHook {
    /// Return the incremental Merkle tree state from the contract.
    async fn tree(&self, _reorg_period: &ReorgPeriod) -> ChainResult<IncrementalMerkleAtBlock> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "tree",
                &[],
                contracts::MERKLE_TREE_HOOK_SOURCE,
            )
            .await?;

        let tree = Self::parse_tree(&result)?;

        Ok(IncrementalMerkleAtBlock {
            tree,
            block_height: None,
        })
    }

    /// Gets the current leaf count of the Merkle tree.
    async fn count(&self, _reorg_period: &ReorgPeriod) -> ChainResult<u32> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "count",
                &[],
                contracts::MERKLE_TREE_HOOK_SOURCE,
            )
            .await?;

        result
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| {
                HyperlaneAeternityError::ContractCallError(format!(
                    "expected integer from count(), got {result}"
                ))
                .into()
            })
    }

    /// Get the latest checkpoint (root + index).
    async fn latest_checkpoint(
        &self,
        _reorg_period: &ReorgPeriod,
    ) -> ChainResult<CheckpointAtBlock> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "latest_checkpoint",
                &[],
                contracts::MERKLE_TREE_HOOK_SOURCE,
            )
            .await?;

        let (root, index) = Self::parse_checkpoint(&result)?;

        Ok(CheckpointAtBlock {
            checkpoint: Checkpoint {
                merkle_tree_hook_address: self.address_h256,
                mailbox_domain: self.domain.id(),
                root,
                index,
            },
            block_height: None,
        })
    }

    /// Get the latest checkpoint at a specific block height.
    async fn latest_checkpoint_at_block(
        &self,
        _height: u64,
    ) -> ChainResult<CheckpointAtBlock> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "latest_checkpoint",
                &[],
                contracts::MERKLE_TREE_HOOK_SOURCE,
            )
            .await?;

        let (root, index) = Self::parse_checkpoint(&result)?;

        Ok(CheckpointAtBlock {
            checkpoint: Checkpoint {
                merkle_tree_hook_address: self.address_h256,
                mailbox_domain: self.domain.id(),
                root,
                index,
            },
            block_height: None,
        })
    }
}

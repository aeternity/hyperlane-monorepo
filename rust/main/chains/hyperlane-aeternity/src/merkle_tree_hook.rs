use async_trait::async_trait;
use num_traits::ToPrimitive;

use hyperlane_core::{
    accumulator::incremental::IncrementalMerkle, ChainResult, Checkpoint, CheckpointAtBlock,
    ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain, HyperlaneProvider,
    IncrementalMerkleAtBlock, MerkleTreeHook, ReorgPeriod, H256,
};

use crate::{
    h256_to_contract_address, AeternityProvider, FateValue, HyperlaneAeternityError,
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
        let contract_address = h256_to_contract_address(locator.address)?;
        Ok(Self {
            domain: provider.domain().clone(),
            provider,
            contract_address,
            address_h256: locator.address,
        })
    }

    /// Parse a FATE tree response into branch array and count.
    ///
    /// The Sophia contract's `tree()` returns a record:
    /// `{ branch: list(bytes(32)), count: int }`
    /// which FATE encodes as a Tuple with fields in alphabetical order.
    fn parse_tree(value: FateValue) -> ChainResult<(IncrementalMerkle, Option<u64>)> {
        let (branch_values, count_value) = match value {
            FateValue::Tuple(fields) if fields.len() == 2 => {
                (fields[0].clone(), fields[1].clone())
            }
            other => {
                return Err(HyperlaneAeternityError::ContractCallError(format!(
                    "expected Tuple(2) from tree(), got {:?}",
                    other
                ))
                .into())
            }
        };

        let branch_list = match branch_values {
            FateValue::List(items) => items,
            other => {
                return Err(HyperlaneAeternityError::ContractCallError(format!(
                    "expected List for tree branch, got {:?}",
                    other
                ))
                .into())
            }
        };

        let mut branch = [H256::zero(); TREE_DEPTH];
        for (i, item) in branch_list.into_iter().enumerate() {
            if i >= TREE_DEPTH {
                break;
            }
            match item {
                FateValue::Bytes(b) if b.len() == 32 => {
                    branch[i] = H256::from_slice(&b);
                }
                other => {
                    return Err(HyperlaneAeternityError::ContractCallError(format!(
                        "expected Bytes(32) for branch node, got {:?}",
                        other
                    ))
                    .into())
                }
            }
        }

        let count = match count_value {
            FateValue::Integer(n) => n.to_usize().ok_or_else(|| {
                HyperlaneAeternityError::ContractCallError(
                    "tree count overflow for usize".into(),
                )
            })?,
            other => {
                return Err(HyperlaneAeternityError::ContractCallError(format!(
                    "expected Integer for tree count, got {:?}",
                    other
                ))
                .into())
            }
        };

        Ok((IncrementalMerkle::new(branch, count), None))
    }

    /// Parse a FATE checkpoint response into root and index.
    ///
    /// The Sophia contract's `latest_checkpoint()` returns:
    /// `{ root: bytes(32), index: int }`
    fn parse_checkpoint(value: FateValue) -> ChainResult<(H256, u32)> {
        let (root_value, index_value) = match value {
            FateValue::Tuple(fields) if fields.len() == 2 => {
                (fields[0].clone(), fields[1].clone())
            }
            other => {
                return Err(HyperlaneAeternityError::ContractCallError(format!(
                    "expected Tuple(2) from latest_checkpoint(), got {:?}",
                    other
                ))
                .into())
            }
        };

        let root = match root_value {
            FateValue::Bytes(b) if b.len() == 32 => H256::from_slice(&b),
            other => {
                return Err(HyperlaneAeternityError::ContractCallError(format!(
                    "expected Bytes(32) for checkpoint root, got {:?}",
                    other
                ))
                .into())
            }
        };

        let index = match index_value {
            FateValue::Integer(n) => n.to_u32().ok_or_else(|| {
                HyperlaneAeternityError::ContractCallError(
                    "checkpoint index overflow for u32".into(),
                )
            })?,
            other => {
                return Err(HyperlaneAeternityError::ContractCallError(format!(
                    "expected Integer for checkpoint index, got {:?}",
                    other
                ))
                .into())
            }
        };

        Ok((root, index))
    }
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
            .call_contract(&self.contract_address, "tree", vec![])
            .await?;

        let (tree, block_height) = Self::parse_tree(result)?;

        Ok(IncrementalMerkleAtBlock {
            tree,
            block_height,
        })
    }

    /// Gets the current leaf count of the Merkle tree.
    async fn count(&self, _reorg_period: &ReorgPeriod) -> ChainResult<u32> {
        let result = self
            .provider
            .call_contract(&self.contract_address, "count", vec![])
            .await?;

        match result {
            FateValue::Integer(n) => n.to_u32().ok_or_else(|| {
                HyperlaneAeternityError::ContractCallError("count overflow for u32".into()).into()
            }),
            other => Err(HyperlaneAeternityError::ContractCallError(format!(
                "expected Integer from count(), got {:?}",
                other
            ))
            .into()),
        }
    }

    /// Get the latest checkpoint (root + index).
    async fn latest_checkpoint(
        &self,
        _reorg_period: &ReorgPeriod,
    ) -> ChainResult<CheckpointAtBlock> {
        let result = self
            .provider
            .call_contract(&self.contract_address, "latest_checkpoint", vec![])
            .await?;

        let (root, index) = Self::parse_checkpoint(result)?;

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
        // AE does not natively support querying at a specific block height
        // via dry-run; fall back to latest state.
        let result = self
            .provider
            .call_contract(&self.contract_address, "latest_checkpoint", vec![])
            .await?;

        let (root, index) = Self::parse_checkpoint(result)?;

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

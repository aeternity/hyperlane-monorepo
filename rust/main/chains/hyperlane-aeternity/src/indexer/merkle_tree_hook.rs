use std::ops::RangeInclusive;

use async_trait::async_trait;

use hyperlane_core::{
    accumulator::incremental::IncrementalMerkle, ChainResult, Checkpoint, CheckpointAtBlock,
    ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain, HyperlaneProvider,
    IncrementalMerkleAtBlock, Indexed, Indexer, LogMeta, MerkleTreeHook, MerkleTreeInsertion,
    ReorgPeriod, SequenceAwareIndexer, H256, H512,
};

use crate::contracts;
use crate::events::{parse_merkle_insertion, ContractLogEntry, INSERTED_INTO_TREE_HASH};
use crate::provider::AeternityProvider;
use crate::types::h256_to_contract_address;

/// Aeternity Merkle Tree Indexer
#[derive(Debug)]
pub struct AeMerkleTreeIndexer {
    provider: AeternityProvider,
    contract_address: String,
    address: H256,
    domain: HyperlaneDomain,
}

impl AeMerkleTreeIndexer {
    /// Creates a new Aeternity Merkle tree indexer.
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address);
        Ok(Self {
            address: locator.address,
            contract_address,
            domain: locator.domain.clone(),
            provider,
        })
    }
}

impl HyperlaneChain for AeMerkleTreeIndexer {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(self.provider.clone())
    }
}

impl HyperlaneContract for AeMerkleTreeIndexer {
    fn address(&self) -> H256 {
        self.address
    }
}

#[async_trait]
impl MerkleTreeHook for AeMerkleTreeIndexer {
    async fn tree(&self, _reorg_period: &ReorgPeriod) -> ChainResult<IncrementalMerkleAtBlock> {
        let tree_data = self
            .provider
            .call_contract(
                &self.contract_address,
                "tree",
                &[],
                &contracts::MERKLE_TREE_HOOK_SOURCE,
            )
            .await?;

        let tree = parse_tree_json(&tree_data);
        let block_height = self.provider.get_finalized_block_number().await?;

        Ok(IncrementalMerkleAtBlock {
            tree,
            block_height: Some(block_height),
        })
    }

    async fn count(&self, _reorg_period: &ReorgPeriod) -> ChainResult<u32> {
        let count = self
            .provider
            .call_contract(
                &self.contract_address,
                "count",
                &[],
                &contracts::MERKLE_TREE_HOOK_SOURCE,
            )
            .await?;

        count.as_u64().map(|n| n as u32).ok_or_else(|| {
            hyperlane_core::ChainCommunicationError::from_other_str("failed to decode count")
        })
    }

    async fn latest_checkpoint(
        &self,
        _reorg_period: &ReorgPeriod,
    ) -> ChainResult<CheckpointAtBlock> {
        let block_height = self.provider.get_finalized_block_number().await?;
        self.latest_checkpoint_at_block(block_height).await
    }

    async fn latest_checkpoint_at_block(
        &self,
        block_height: u64,
    ) -> ChainResult<CheckpointAtBlock> {
        let checkpoint_data = self
            .provider
            .call_contract(
                &self.contract_address,
                "latest_checkpoint",
                &[],
                &contracts::MERKLE_TREE_HOOK_SOURCE,
            )
            .await?;

        let (root, index) = parse_checkpoint_json(&checkpoint_data);

        let domain_data = self
            .provider
            .call_contract(
                &self.contract_address,
                "local_domain",
                &[],
                &contracts::MERKLE_TREE_HOOK_SOURCE,
            )
            .await?;

        let mailbox_domain = domain_data.as_u64().map(|n| n as u32).unwrap_or(self.domain.id());

        Ok(CheckpointAtBlock {
            checkpoint: Checkpoint {
                merkle_tree_hook_address: self.address,
                mailbox_domain,
                root,
                index,
            },
            block_height: Some(block_height),
        })
    }
}

/// Parse compiler-decoded tree JSON into `IncrementalMerkle`.
fn parse_tree_json(value: &serde_json::Value) -> IncrementalMerkle {
    let mut branch = [H256::zero(); 32];
    let mut count = 0usize;

    if let Some(obj) = value.as_object() {
        if let Some(branch_val) = obj.get("branch") {
            // The AE compiler returns maps as arrays of [key, value] pairs,
            // e.g. [[0, "#abcd..."], [1, "#ef01..."]].
            if let Some(arr) = branch_val.as_array() {
                for pair in arr {
                    if let Some(kv) = pair.as_array() {
                        if kv.len() == 2 {
                            if let (Some(idx), Some(hex_str)) =
                                (kv[0].as_u64().map(|n| n as usize), kv[1].as_str())
                            {
                                if idx < 32 {
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
                }
            } else if let Some(branch_map) = branch_val.as_object() {
                for (key, val) in branch_map {
                    if let (Ok(idx), Some(hex_str)) = (key.parse::<usize>(), val.as_str()) {
                        if idx < 32 {
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
        }
        if let Some(c) = obj.get("count").and_then(|v| v.as_u64()) {
            count = c as usize;
        }
    }

    IncrementalMerkle::new(branch, count)
}

/// Parse compiler-decoded checkpoint tuple JSON into `(root, index)`.
fn parse_checkpoint_json(value: &serde_json::Value) -> (H256, u32) {
    if let Some(arr) = value.as_array() {
        if arr.len() == 2 {
            let root = arr[0]
                .as_str()
                .and_then(|s| {
                    let clean = s.trim_start_matches('#');
                    hex::decode(clean).ok()
                })
                .filter(|b| b.len() == 32)
                .map(|b| H256::from_slice(&b))
                .unwrap_or_default();
            let index = arr[1].as_u64().unwrap_or(0) as u32;
            return (root, index);
        }
    }
    (H256::zero(), 0)
}

#[async_trait]
impl Indexer<MerkleTreeInsertion> for AeMerkleTreeIndexer {
    async fn fetch_logs_in_range(
        &self,
        range: RangeInclusive<u32>,
    ) -> ChainResult<Vec<(Indexed<MerkleTreeInsertion>, LogMeta)>> {
        let from = *range.start() as u64;
        let to = *range.end() as u64;
        let logs = self
            .provider
            .fetch_logs_in_range(&self.contract_address, from, to)
            .await?;

        let mut result = Vec::new();
        for (mdw_entry, _meta) in &logs {
            let entry = ContractLogEntry::from(mdw_entry);
            if entry.event_hash != *INSERTED_INTO_TREE_HASH {
                continue;
            }
            match parse_merkle_insertion(&entry) {
                Ok(Some(parsed)) => result.push(parsed),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "failed to parse merkle insertion event");
                }
            }
        }
        Ok(result)
    }

    async fn get_finalized_block_number(&self) -> ChainResult<u32> {
        let block = self.provider.get_finalized_block_number().await?;
        Ok(block as u32)
    }

    async fn fetch_logs_by_tx_hash(
        &self,
        tx_hash: H512,
    ) -> ChainResult<Vec<(Indexed<MerkleTreeInsertion>, LogMeta)>> {
        let logs = self
            .provider
            .fetch_logs_in_range(&self.contract_address, 0, u64::MAX)
            .await?;

        let tx_hash_hex = format!("{tx_hash:x}");
        let mut result = Vec::new();
        for (mdw_entry, _meta) in &logs {
            let entry = ContractLogEntry::from(mdw_entry);
            if !entry.call_tx_hash.contains(&tx_hash_hex) {
                continue;
            }
            if entry.event_hash != *INSERTED_INTO_TREE_HASH {
                continue;
            }
            match parse_merkle_insertion(&entry) {
                Ok(Some(parsed)) => result.push(parsed),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "failed to parse merkle insertion by tx");
                }
            }
        }
        Ok(result)
    }
}

#[async_trait]
impl SequenceAwareIndexer<MerkleTreeInsertion> for AeMerkleTreeIndexer {
    async fn latest_sequence_count_and_tip(&self) -> ChainResult<(Option<u32>, u32)> {
        // Get the tip FIRST, then the count. AE micro blocks can update
        // contract state before the key block height advances, so fetching
        // count first could yield a sequence that doesn't exist yet at the
        // reported tip, causing the forward cursor to loop forever.
        let tip = self.provider.get_finalized_block_number().await? as u32;

        let count = self
            .provider
            .call_contract(
                &self.contract_address,
                "count",
                &[],
                &contracts::MERKLE_TREE_HOOK_SOURCE,
            )
            .await?;

        let sequence = count.as_u64().map(|n| n as u32).unwrap_or(0);
        Ok((Some(sequence), tip))
    }
}

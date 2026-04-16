use std::ops::RangeInclusive;

use async_trait::async_trait;
use num_traits::ToPrimitive;

use hyperlane_core::{
    accumulator::incremental::IncrementalMerkle, ChainResult, Checkpoint, CheckpointAtBlock,
    ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain, HyperlaneProvider,
    IncrementalMerkleAtBlock, Indexed, Indexer, LogMeta, MerkleTreeHook, MerkleTreeInsertion,
    ReorgPeriod, SequenceAwareIndexer, H256, H512,
};

use crate::events::{parse_merkle_insertion, ContractLogEntry, INSERTED_INTO_TREE_HASH};
use crate::provider::{AeternityProvider, FateValue};
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
            .call_contract(&self.contract_address, "tree", vec![])
            .await?;

        let (branch, count) = match tree_data {
            FateValue::Tuple(fields) if fields.len() == 2 => {
                let branch_list = match &fields[0] {
                    FateValue::List(items) => {
                        let mut branch = [H256::zero(); 32];
                        for (i, item) in items.iter().enumerate() {
                            if i >= 32 { break; }
                            if let FateValue::Bytes(b) = item {
                                if b.len() == 32 {
                                    branch[i] = H256::from_slice(b);
                                }
                            }
                        }
                        branch
                    }
                    _ => [H256::zero(); 32],
                };
                let count = match &fields[1] {
                    FateValue::Integer(n) => n.to_usize().unwrap_or(0),
                    _ => 0,
                };
                (branch_list, count)
            }
            _ => ([H256::zero(); 32], 0),
        };

        let tree = IncrementalMerkle::new(branch, count);
        let block_height = self.provider.get_finalized_block_number().await?;

        Ok(IncrementalMerkleAtBlock {
            tree,
            block_height: Some(block_height),
        })
    }

    async fn count(&self, _reorg_period: &ReorgPeriod) -> ChainResult<u32> {
        let count = self
            .provider
            .call_contract(&self.contract_address, "count", vec![])
            .await?;
        match count {
            FateValue::Integer(n) => n.to_u32().ok_or_else(|| {
                hyperlane_core::ChainCommunicationError::from_other_str("failed to decode count")
            }),
            _ => Err(hyperlane_core::ChainCommunicationError::from_other_str(
                "unexpected type from count()",
            )),
        }
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
            .call_contract(&self.contract_address, "latest_checkpoint", vec![])
            .await?;

        let (root, index) = match checkpoint_data {
            FateValue::Tuple(fields) if fields.len() == 2 => {
                let root = match &fields[0] {
                    FateValue::Bytes(b) if b.len() == 32 => H256::from_slice(b),
                    _ => H256::zero(),
                };
                let index = match &fields[1] {
                    FateValue::Integer(n) => n.to_u32().unwrap_or(0),
                    _ => 0,
                };
                (root, index)
            }
            _ => (H256::zero(), 0u32),
        };

        let domain_data = self
            .provider
            .call_contract(&self.contract_address, "local_domain", vec![])
            .await?;

        let mailbox_domain = match domain_data {
            FateValue::Integer(n) => n.to_u32().unwrap_or(self.domain.id()),
            _ => self.domain.id(),
        };

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
        let count = self
            .provider
            .call_contract(&self.contract_address, "count", vec![])
            .await?;

        let sequence = match count {
            FateValue::Integer(n) => n.to_u32().unwrap_or(0),
            _ => 0,
        };
        let tip = self.provider.get_finalized_block_number().await? as u32;
        Ok((Some(sequence), tip))
    }
}

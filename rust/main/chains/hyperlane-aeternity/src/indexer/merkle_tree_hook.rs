use std::ops::RangeInclusive;

use async_trait::async_trait;

use hyperlane_core::{
    accumulator::incremental::IncrementalMerkle, ChainResult, Checkpoint, CheckpointAtBlock,
    ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain, HyperlaneProvider,
    IncrementalMerkleAtBlock, Indexed, Indexer, LogMeta, MerkleTreeHook, MerkleTreeInsertion,
    ReorgPeriod, SequenceAwareIndexer, H256, H512,
};

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

        let (branch, count) = tree_data.as_merkle_tree().map_err(|e| {
            hyperlane_core::ChainCommunicationError::from_other_str(&format!(
                "failed to decode merkle tree: {e}"
            ))
        })?;

        let tree = IncrementalMerkle { branch, count };
        let block_height = self.provider.get_finalized_block_number().await? as u64;

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
        count.as_u32().ok_or_else(|| {
            hyperlane_core::ChainCommunicationError::from_other_str("failed to decode count")
        })
    }

    async fn latest_checkpoint(
        &self,
        _reorg_period: &ReorgPeriod,
    ) -> ChainResult<CheckpointAtBlock> {
        let block_height = self.provider.get_finalized_block_number().await? as u64;
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

        let (root, index) = checkpoint_data.as_checkpoint().map_err(|e| {
            hyperlane_core::ChainCommunicationError::from_other_str(&format!(
                "failed to decode checkpoint: {e}"
            ))
        })?;

        let domain_data = self
            .provider
            .call_contract(&self.contract_address, "local_domain", vec![])
            .await?;

        let mailbox_domain = domain_data.as_u32().unwrap_or(self.domain.id());

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
        let logs = self
            .provider
            .fetch_logs_in_range(&self.contract_address, range)
            .await?;

        let mut result = Vec::new();
        for log in &logs {
            if log.event_hash != *INSERTED_INTO_TREE_HASH {
                continue;
            }
            match parse_merkle_insertion(log) {
                Ok(Some(entry)) => result.push(entry),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "failed to parse merkle insertion event");
                }
            }
        }
        Ok(result)
    }

    async fn get_finalized_block_number(&self) -> ChainResult<u32> {
        self.provider.get_finalized_block_number().await
    }

    async fn fetch_logs_by_tx_hash(
        &self,
        tx_hash: H512,
    ) -> ChainResult<Vec<(Indexed<MerkleTreeInsertion>, LogMeta)>> {
        let logs = self
            .provider
            .fetch_logs_in_range(&self.contract_address, 0..=u32::MAX)
            .await?;

        let tx_hash_hex = format!("{tx_hash:x}");
        let mut result = Vec::new();
        for log in &logs {
            if !log.call_tx_hash.contains(&tx_hash_hex) {
                continue;
            }
            if log.event_hash != *INSERTED_INTO_TREE_HASH {
                continue;
            }
            match parse_merkle_insertion(log) {
                Ok(Some(entry)) => result.push(entry),
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

        let sequence = count.as_u32().unwrap_or(0);
        let tip = self.provider.get_finalized_block_number().await?;
        Ok((Some(sequence), tip))
    }
}

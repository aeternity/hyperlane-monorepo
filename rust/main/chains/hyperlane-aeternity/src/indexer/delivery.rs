use std::ops::RangeInclusive;

use async_trait::async_trait;

use hyperlane_core::{
    ChainResult, ContractLocator, Indexed, Indexer, LogMeta, SequenceAwareIndexer, H256, H512,
};

use crate::contracts;
use crate::events::{parse_delivery_event, ContractLogEntry, PROCESS_ID_EVENT_HASH};
use crate::provider::AeternityProvider;
use crate::types::h256_to_contract_address;

/// Aeternity Delivery Indexer
#[derive(Debug)]
pub struct AeDeliveryIndexer {
    provider: AeternityProvider,
    contract_address: String,
}

impl AeDeliveryIndexer {
    /// Creates a new Aeternity delivery indexer.
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address);
        Ok(Self {
            provider,
            contract_address,
        })
    }

    /// Get the Mailbox deployment block to use as indexing start.
    /// Falls back to 0 if the query fails (for contracts without deployed_block).
    pub async fn get_start_block(&self) -> u64 {
        match self
            .provider
            .call_contract(
                &self.contract_address,
                "deployed_block",
                &[],
                &contracts::MAILBOX_SOURCE,
            )
            .await
        {
            Ok(val) => val.as_u64().unwrap_or(0),
            Err(_) => 0,
        }
    }
}

#[async_trait]
impl Indexer<H256> for AeDeliveryIndexer {
    async fn fetch_logs_in_range(
        &self,
        range: RangeInclusive<u32>,
    ) -> ChainResult<Vec<(Indexed<H256>, LogMeta)>> {
        let from = *range.start() as u64;
        let to = *range.end() as u64;
        let logs = self
            .provider
            .fetch_logs_in_range(&self.contract_address, from, to)
            .await?;

        let mut result = Vec::new();
        for (mdw_entry, _meta) in &logs {
            let entry = ContractLogEntry::from(mdw_entry);
            if entry.event_hash != *PROCESS_ID_EVENT_HASH {
                continue;
            }
            match parse_delivery_event(&entry) {
                Ok(Some(parsed)) => result.push(parsed),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "failed to parse delivery event");
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
    ) -> ChainResult<Vec<(Indexed<H256>, LogMeta)>> {
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
            if entry.event_hash != *PROCESS_ID_EVENT_HASH {
                continue;
            }
            match parse_delivery_event(&entry) {
                Ok(Some(parsed)) => result.push(parsed),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "failed to parse delivery event by tx");
                }
            }
        }
        Ok(result)
    }
}

#[async_trait]
impl SequenceAwareIndexer<H256> for AeDeliveryIndexer {
    async fn latest_sequence_count_and_tip(&self) -> ChainResult<(Option<u32>, u32)> {
        let tip = self.provider.get_finalized_block_number().await? as u32;

        let count = self
            .provider
            .call_contract(
                &self.contract_address,
                "nonce",
                &[],
                &contracts::MAILBOX_SOURCE,
            )
            .await?;

        let sequence = count.as_u64().map(|n| n as u32).unwrap_or(0);
        Ok((Some(sequence), tip))
    }
}

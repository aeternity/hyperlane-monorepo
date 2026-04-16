use std::ops::RangeInclusive;

use async_trait::async_trait;

use hyperlane_core::{
    ChainResult, ContractLocator, Indexed, Indexer, LogMeta, SequenceAwareIndexer, H256, H512,
};

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
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address);
        Ok(Self {
            provider,
            contract_address,
        })
    }
}

#[async_trait]
impl Indexer<H256> for AeDeliveryIndexer {
    async fn fetch_logs_in_range(
        &self,
        range: RangeInclusive<u32>,
    ) -> ChainResult<Vec<(Indexed<H256>, LogMeta)>> {
        let logs = self
            .provider
            .fetch_logs_in_range(&self.contract_address, range)
            .await?;

        let mut result = Vec::new();
        for log in &logs {
            if log.event_hash != *PROCESS_ID_EVENT_HASH {
                continue;
            }
            match parse_delivery_event(log) {
                Ok(Some(entry)) => result.push(entry),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "failed to parse delivery event");
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
    ) -> ChainResult<Vec<(Indexed<H256>, LogMeta)>> {
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
            if log.event_hash != *PROCESS_ID_EVENT_HASH {
                continue;
            }
            match parse_delivery_event(log) {
                Ok(Some(entry)) => result.push(entry),
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
        let count = self
            .provider
            .call_contract(&self.contract_address, "processed_count", vec![])
            .await?;

        let sequence = count.as_u32().unwrap_or(0);
        let tip = self.provider.get_finalized_block_number().await?;
        Ok((Some(sequence), tip))
    }
}

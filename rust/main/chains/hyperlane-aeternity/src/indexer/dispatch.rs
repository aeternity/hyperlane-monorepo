use std::ops::RangeInclusive;

use async_trait::async_trait;

use hyperlane_core::{
    ChainResult, ContractLocator, HyperlaneMessage, Indexed, Indexer, LogMeta,
    SequenceAwareIndexer, H512,
};

use crate::events::{parse_dispatch_event, ContractLogEntry, DISPATCH_EVENT_HASH};
use crate::provider::AeternityProvider;
use crate::types::h256_to_contract_address;

/// Aeternity Dispatch Indexer
#[derive(Debug)]
pub struct AeDispatchIndexer {
    provider: AeternityProvider,
    contract_address: String,
}

impl AeDispatchIndexer {
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address);
        Ok(Self {
            provider,
            contract_address,
        })
    }
}

#[async_trait]
impl Indexer<HyperlaneMessage> for AeDispatchIndexer {
    async fn fetch_logs_in_range(
        &self,
        range: RangeInclusive<u32>,
    ) -> ChainResult<Vec<(Indexed<HyperlaneMessage>, LogMeta)>> {
        let logs = self
            .provider
            .fetch_logs_in_range(&self.contract_address, range)
            .await?;

        let mut result = Vec::new();
        for log in &logs {
            if log.event_hash != *DISPATCH_EVENT_HASH {
                continue;
            }
            match parse_dispatch_event(log) {
                Ok(Some(entry)) => result.push(entry),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "failed to parse dispatch event");
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
    ) -> ChainResult<Vec<(Indexed<HyperlaneMessage>, LogMeta)>> {
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
            if log.event_hash != *DISPATCH_EVENT_HASH {
                continue;
            }
            match parse_dispatch_event(log) {
                Ok(Some(entry)) => result.push(entry),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "failed to parse dispatch event by tx");
                }
            }
        }
        Ok(result)
    }
}

#[async_trait]
impl SequenceAwareIndexer<HyperlaneMessage> for AeDispatchIndexer {
    async fn latest_sequence_count_and_tip(&self) -> ChainResult<(Option<u32>, u32)> {
        let nonce = self
            .provider
            .call_contract(&self.contract_address, "nonce", vec![])
            .await?;

        let sequence = nonce.as_u32().unwrap_or(0);
        let tip = self.provider.get_finalized_block_number().await?;
        Ok((Some(sequence), tip))
    }
}

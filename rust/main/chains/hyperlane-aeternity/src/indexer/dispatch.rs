use std::ops::RangeInclusive;

use async_trait::async_trait;

use hyperlane_core::{
    ChainResult, ContractLocator, HyperlaneMessage, Indexed, Indexer, LogMeta,
    SequenceAwareIndexer, H512,
};

use crate::contracts;
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
    /// Creates a new Aeternity dispatch indexer.
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
        let from = *range.start() as u64;
        let to = *range.end() as u64;
        let logs = self
            .provider
            .fetch_logs_in_range(&self.contract_address, from, to)
            .await?;

        let mut result = Vec::new();
        for (mdw_entry, _meta) in &logs {
            let entry = ContractLogEntry::from(mdw_entry);
            if entry.event_hash != *DISPATCH_EVENT_HASH {
                continue;
            }
            match parse_dispatch_event(&entry) {
                Ok(Some(parsed)) => result.push(parsed),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "failed to parse dispatch event");
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
    ) -> ChainResult<Vec<(Indexed<HyperlaneMessage>, LogMeta)>> {
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
            if entry.event_hash != *DISPATCH_EVENT_HASH {
                continue;
            }
            match parse_dispatch_event(&entry) {
                Ok(Some(parsed)) => result.push(parsed),
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
        let tip = self.provider.get_finalized_block_number().await? as u32;

        let nonce = self
            .provider
            .call_contract(
                &self.contract_address,
                "nonce",
                &[],
                &contracts::MAILBOX_SOURCE,
            )
            .await?;

        let sequence = nonce.as_u64().map(|n| n as u32).unwrap_or(0);
        Ok((Some(sequence), tip))
    }
}

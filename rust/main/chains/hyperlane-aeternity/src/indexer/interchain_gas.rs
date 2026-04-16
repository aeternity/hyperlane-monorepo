use std::ops::RangeInclusive;

use async_trait::async_trait;

use hyperlane_core::{
    ChainResult, ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneProvider, Indexed, Indexer, InterchainGasPaymaster, InterchainGasPayment, LogMeta,
    SequenceAwareIndexer, H256, H512,
};

use crate::events::{parse_gas_payment, ContractLogEntry, GAS_PAYMENT_HASH};
use crate::provider::AeternityProvider;
use crate::types::h256_to_contract_address;

/// Aeternity Interchain Gas Indexer
#[derive(Debug)]
pub struct AeIgpIndexer {
    provider: AeternityProvider,
    contract_address: String,
    address: H256,
    domain: HyperlaneDomain,
}

impl AeIgpIndexer {
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address);
        Ok(Self {
            contract_address,
            address: locator.address,
            provider,
            domain: locator.domain.clone(),
        })
    }
}

impl HyperlaneChain for AeIgpIndexer {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(self.provider.clone())
    }
}

impl HyperlaneContract for AeIgpIndexer {
    fn address(&self) -> H256 {
        self.address
    }
}

#[async_trait]
impl InterchainGasPaymaster for AeIgpIndexer {}

#[async_trait]
impl Indexer<InterchainGasPayment> for AeIgpIndexer {
    async fn fetch_logs_in_range(
        &self,
        range: RangeInclusive<u32>,
    ) -> ChainResult<Vec<(Indexed<InterchainGasPayment>, LogMeta)>> {
        let logs = self
            .provider
            .fetch_logs_in_range(&self.contract_address, range)
            .await?;

        let mut result = Vec::new();
        for log in &logs {
            if log.event_hash != *GAS_PAYMENT_HASH {
                continue;
            }
            match parse_gas_payment(log) {
                Ok(Some(entry)) => result.push(entry),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "failed to parse gas payment event");
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
    ) -> ChainResult<Vec<(Indexed<InterchainGasPayment>, LogMeta)>> {
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
            if log.event_hash != *GAS_PAYMENT_HASH {
                continue;
            }
            match parse_gas_payment(log) {
                Ok(Some(entry)) => result.push(entry),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "failed to parse gas payment by tx");
                }
            }
        }
        Ok(result)
    }
}

#[async_trait]
impl SequenceAwareIndexer<InterchainGasPayment> for AeIgpIndexer {
    async fn latest_sequence_count_and_tip(&self) -> ChainResult<(Option<u32>, u32)> {
        let count = self
            .provider
            .call_contract(&self.contract_address, "sequence", vec![])
            .await?;

        let sequence = count.as_u32().unwrap_or(0);
        let tip = self.provider.get_finalized_block_number().await?;
        Ok((Some(sequence), tip))
    }
}

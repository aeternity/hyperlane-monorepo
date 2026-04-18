use async_trait::async_trait;

use hyperlane_core::{
    AggregationIsm, ChainResult, ContractLocator, Encode, HyperlaneChain, HyperlaneContract,
    HyperlaneDomain, HyperlaneMessage, HyperlaneProvider, InterchainSecurityModule, Metadata,
    ModuleType, H256, U256,
};

use crate::{
    ae_address_to_h256, contracts, h256_to_contract_address, AeternityProvider,
    HyperlaneAeternityError,
};

use super::interchain_security_module::json_to_module_type;

/// Aeternity Aggregation ISM
///
/// Scaffolded for future use — the Sophia `AggregationIsm` contract is not yet
/// deployed, but the Rust wrapper is ready for when it is.
#[derive(Debug)]
pub struct AeAggregationIsm {
    provider: AeternityProvider,
    contract_address: String,
    address_h256: H256,
    domain: HyperlaneDomain,
}

impl AeAggregationIsm {
    /// Creates a new Aeternity AggregationIsm instance
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address);
        Ok(Self {
            domain: provider.domain().clone(),
            provider,
            contract_address,
            address_h256: locator.address,
        })
    }
}

impl HyperlaneContract for AeAggregationIsm {
    fn address(&self) -> H256 {
        self.address_h256
    }
}

impl HyperlaneChain for AeAggregationIsm {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(self.provider.clone())
    }
}

#[async_trait]
impl InterchainSecurityModule for AeAggregationIsm {
    async fn module_type(&self) -> ChainResult<ModuleType> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "module_type",
                &[],
                contracts::AGGREGATION_ISM_SOURCE,
            )
            .await?;

        json_to_module_type(&result)
    }

    async fn dry_run_verify(
        &self,
        _message: &HyperlaneMessage,
        _metadata: &Metadata,
    ) -> ChainResult<Option<U256>> {
        Ok(None)
    }
}

#[async_trait]
impl AggregationIsm for AeAggregationIsm {
    /// Returns the `m` sub-ISMs and `n` threshold needed for n-of-m verification.
    ///
    /// Calls Sophia entrypoint:
    ///   `modules_and_threshold(message: bytes()) : list(address) * int`
    async fn modules_and_threshold(
        &self,
        message: &HyperlaneMessage,
    ) -> ChainResult<(Vec<H256>, u8)> {
        let message_hex = format!("#{}", hex::encode(message.to_vec()));

        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "modules_and_threshold",
                &[message_hex],
                contracts::AGGREGATION_ISM_SOURCE,
            )
            .await?;

        let arr = result.as_array().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected tuple array from modules_and_threshold(), got {result}"
            ))
        })?;

        if arr.len() != 2 {
            return Err(HyperlaneAeternityError::ContractCallError(format!(
                "expected 2-element tuple, got {} elements",
                arr.len()
            ))
            .into());
        }

        let modules_arr = arr[0].as_array().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected list for modules, got {}",
                arr[0]
            ))
        })?;

        let mut modules = Vec::with_capacity(modules_arr.len());
        for item in modules_arr {
            let addr_str = item.as_str().ok_or_else(|| {
                HyperlaneAeternityError::ContractCallError(format!(
                    "expected address string for module, got {item}"
                ))
            })?;
            modules.push(ae_address_to_h256(addr_str)?);
        }

        let threshold = arr[1].as_u64().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected integer for threshold, got {}",
                arr[1]
            ))
        })? as u8;

        Ok((modules, threshold))
    }
}

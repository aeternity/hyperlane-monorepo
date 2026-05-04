use async_trait::async_trait;

use hyperlane_core::{
    AggregationIsm, ChainResult, ContractLocator, HyperlaneChain, HyperlaneContract,
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
                &contracts::AGGREGATION_ISM_SOURCE,
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
    /// Returns the sub-ISMs and threshold needed for n-of-m verification.
    ///
    /// Calls Sophia entrypoints:
    ///   `get_modules() : list(IInterchainSecurityModule)`
    ///   `get_threshold() : int`
    async fn modules_and_threshold(
        &self,
        _message: &HyperlaneMessage,
    ) -> ChainResult<(Vec<H256>, u8)> {
        let modules_result = self
            .provider
            .call_contract(
                &self.contract_address,
                "get_modules",
                &[],
                &contracts::AGGREGATION_ISM_SOURCE,
            )
            .await?;

        let modules_arr = modules_result.as_array().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected list from get_modules(), got {modules_result}"
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

        let threshold_result = self
            .provider
            .call_contract(
                &self.contract_address,
                "get_threshold",
                &[],
                &contracts::AGGREGATION_ISM_SOURCE,
            )
            .await?;

        let threshold = threshold_result.as_u64().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected integer from get_threshold(), got {threshold_result}"
            ))
        })? as u8;

        Ok((modules, threshold))
    }
}

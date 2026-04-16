use async_trait::async_trait;
use num_traits::ToPrimitive;

use hyperlane_core::{
    AggregationIsm, ChainResult, ContractLocator, Encode, HyperlaneChain, HyperlaneContract,
    HyperlaneDomain, HyperlaneMessage, HyperlaneProvider, InterchainSecurityModule, Metadata,
    ModuleType, H256, U256,
};

use crate::{
    contract_address_to_h256, h256_to_contract_address, AeternityProvider, FateValue,
    HyperlaneAeternityError,
};

use super::interchain_security_module::fate_to_module_type;

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
        let contract_address = h256_to_contract_address(locator.address)?;
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
    /// Returns the module type of this ISM.
    async fn module_type(&self) -> ChainResult<ModuleType> {
        let result = self
            .provider
            .call_contract(&self.contract_address, "module_type", vec![])
            .await?;

        fate_to_module_type(result)
    }

    /// Dry runs the `verify()` ISM call.
    ///
    /// Returns `None` — Aeternity handles verification on-chain.
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
    ///   `modules_and_threshold(message: bytes) : { modules: list(IInterchainSecurityModule), threshold: int }`
    ///
    /// Each module address is converted from `ct_...` format to H256.
    async fn modules_and_threshold(
        &self,
        message: &HyperlaneMessage,
    ) -> ChainResult<(Vec<H256>, u8)> {
        let message_bytes = message.to_vec();

        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "modules_and_threshold",
                vec![FateValue::Bytes(message_bytes)],
            )
            .await?;

        let (modules_value, threshold_value) = match result {
            FateValue::Tuple(fields) if fields.len() == 2 => {
                (fields[0].clone(), fields[1].clone())
            }
            other => {
                return Err(HyperlaneAeternityError::ContractCallError(format!(
                    "expected Tuple(2) from modules_and_threshold(), got {:?}",
                    other
                ))
                .into())
            }
        };

        let modules = match modules_value {
            FateValue::List(items) => {
                let mut addrs = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        FateValue::Address(addr) => {
                            addrs.push(contract_address_to_h256(&addr)?);
                        }
                        other => {
                            return Err(HyperlaneAeternityError::ContractCallError(
                                format!("expected Address for module, got {:?}", other),
                            )
                            .into())
                        }
                    }
                }
                addrs
            }
            other => {
                return Err(HyperlaneAeternityError::ContractCallError(format!(
                    "expected List for modules, got {:?}",
                    other
                ))
                .into())
            }
        };

        let threshold = match threshold_value {
            FateValue::Integer(n) => n.to_u8().ok_or_else(|| {
                HyperlaneAeternityError::ContractCallError("threshold overflow for u8".into())
            })?,
            other => {
                return Err(HyperlaneAeternityError::ContractCallError(format!(
                    "expected Integer for threshold, got {:?}",
                    other
                ))
                .into())
            }
        };

        Ok((modules, threshold))
    }
}

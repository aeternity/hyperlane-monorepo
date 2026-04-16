use async_trait::async_trait;
use num_traits::ToPrimitive;

use hyperlane_core::{
    ChainResult, ContractLocator, Encode, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneMessage, HyperlaneProvider, InterchainSecurityModule, Metadata, ModuleType,
    MultisigIsm, H256, U256,
};

use crate::{
    h256_to_contract_address, AeternityProvider, FateValue, HyperlaneAeternityError,
};

use super::interchain_security_module::fate_to_module_type;

/// Aeternity Multisig ISM
///
/// Wraps `MessageIdMultisigIsm.aes` — returns the set of validators and
/// threshold needed to verify a message.
#[derive(Debug)]
pub struct AeMultisigIsm {
    provider: AeternityProvider,
    contract_address: String,
    address_h256: H256,
    domain: HyperlaneDomain,
}

impl AeMultisigIsm {
    /// Creates a new Aeternity MultisigIsm instance
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

impl HyperlaneContract for AeMultisigIsm {
    fn address(&self) -> H256 {
        self.address_h256
    }
}

impl HyperlaneChain for AeMultisigIsm {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(self.provider.clone())
    }
}

#[async_trait]
impl InterchainSecurityModule for AeMultisigIsm {
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
impl MultisigIsm for AeMultisigIsm {
    /// Returns the validator and threshold needed to verify message.
    ///
    /// Calls Sophia entrypoint:
    ///   `validators_and_threshold(message: bytes) : { validators: list(bytes(20)), threshold: int }`
    ///
    /// Each 20-byte validator address is left-padded with 12 zero bytes to produce H256.
    async fn validators_and_threshold(
        &self,
        message: &HyperlaneMessage,
    ) -> ChainResult<(Vec<H256>, u8)> {
        let message_bytes = message.to_vec();

        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "validators_and_threshold",
                vec![FateValue::Bytes(message_bytes)],
            )
            .await?;

        let (validators_value, threshold_value) = match result {
            FateValue::Tuple(fields) if fields.len() == 2 => {
                (fields[0].clone(), fields[1].clone())
            }
            other => {
                return Err(HyperlaneAeternityError::ContractCallError(format!(
                    "expected Tuple(2) from validators_and_threshold(), got {:?}",
                    other
                ))
                .into())
            }
        };

        let validators = match validators_value {
            FateValue::List(items) => {
                let mut addrs = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        FateValue::Bytes(b) if b.len() == 20 => {
                            let mut h256_bytes = [0u8; 32];
                            h256_bytes[12..].copy_from_slice(&b);
                            addrs.push(H256::from(h256_bytes));
                        }
                        other => {
                            return Err(HyperlaneAeternityError::ContractCallError(
                                format!(
                                    "expected Bytes(20) for validator address, got {:?}",
                                    other
                                ),
                            )
                            .into())
                        }
                    }
                }
                addrs
            }
            other => {
                return Err(HyperlaneAeternityError::ContractCallError(format!(
                    "expected List for validators, got {:?}",
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

        Ok((validators, threshold))
    }
}

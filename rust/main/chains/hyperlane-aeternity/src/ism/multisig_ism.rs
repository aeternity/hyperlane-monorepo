use async_trait::async_trait;

use hyperlane_core::{
    ChainResult, ContractLocator, Encode, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneMessage, HyperlaneProvider, InterchainSecurityModule, Metadata, ModuleType,
    MultisigIsm, H256, U256,
};

use crate::{
    contracts, h256_to_contract_address, AeternityProvider, HyperlaneAeternityError,
};

use super::interchain_security_module::json_to_module_type;

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
    async fn module_type(&self) -> ChainResult<ModuleType> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "module_type",
                &[],
                contracts::MULTISIG_ISM_SOURCE,
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
impl MultisigIsm for AeMultisigIsm {
    /// Returns the validators and threshold needed to verify a message.
    ///
    /// Calls Sophia entrypoint:
    ///   `validators_and_threshold(message: bytes()) : list(bytes(20)) * int`
    async fn validators_and_threshold(
        &self,
        message: &HyperlaneMessage,
    ) -> ChainResult<(Vec<H256>, u8)> {
        let message_hex = format!("#{}", hex::encode(message.to_vec()));

        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "validators_and_threshold",
                &[message_hex],
                contracts::MULTISIG_ISM_SOURCE,
            )
            .await?;

        let arr = result.as_array().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected tuple array from validators_and_threshold(), got {result}"
            ))
        })?;

        if arr.len() != 2 {
            return Err(HyperlaneAeternityError::ContractCallError(format!(
                "expected 2-element tuple, got {} elements",
                arr.len()
            ))
            .into());
        }

        let validators_arr = arr[0].as_array().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected list for validators, got {}",
                arr[0]
            ))
        })?;

        let mut validators = Vec::with_capacity(validators_arr.len());
        for item in validators_arr {
            let hex_str = item.as_str().ok_or_else(|| {
                HyperlaneAeternityError::ContractCallError(format!(
                    "expected hex string for validator, got {item}"
                ))
            })?;
            let hex_clean = hex_str.trim_start_matches('#');
            let bytes = hex::decode(hex_clean).map_err(|e| {
                HyperlaneAeternityError::ContractCallError(format!(
                    "invalid hex for validator: {e}"
                ))
            })?;
            if bytes.len() == 20 {
                let mut h256_bytes = [0u8; 32];
                h256_bytes[12..].copy_from_slice(&bytes);
                validators.push(H256::from(h256_bytes));
            }
        }

        let threshold = arr[1].as_u64().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected integer for threshold, got {}",
                arr[1]
            ))
        })? as u8;

        Ok((validators, threshold))
    }
}

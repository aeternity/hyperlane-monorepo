use async_trait::async_trait;

use hyperlane_core::{
    ChainResult, ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneProvider, InterchainGasPaymaster, TxOutcome, H256, U256,
};

use crate::{contracts, h256_to_contract_address, AeternityProvider, HyperlaneAeternityError};

/// Aeternity Interchain Gas Paymaster
#[derive(Debug)]
pub struct AeInterchainGasPaymaster {
    provider: AeternityProvider,
    contract_address: String,
    address_h256: H256,
    domain: HyperlaneDomain,
}

impl AeInterchainGasPaymaster {
    /// Creates a new Aeternity InterchainGasPaymaster instance
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address);
        Ok(Self {
            domain: provider.domain().clone(),
            provider,
            contract_address,
            address_h256: locator.address,
        })
    }

    /// Quote the gas payment required for a message to a destination domain.
    ///
    /// Calls Sophia entrypoint: `quote_gas_payment(dest_domain: int, gas_amount: int) : int`
    pub async fn quote_gas_payment(
        &self,
        destination_domain: u32,
        gas_amount: U256,
    ) -> ChainResult<U256> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "quote_gas_payment",
                &[
                    destination_domain.to_string(),
                    gas_amount.as_u128().to_string(),
                ],
                &contracts::IGP_SOURCE,
            )
            .await?;

        // Handle both u64 and string returns for large AE amounts (18 decimals)
        if let Some(val) = result.as_u64() {
            return Ok(U256::from(val));
        }
        if let Some(s) = result.as_str() {
            return U256::from_dec_str(s).map_err(|e| {
                HyperlaneAeternityError::ContractCallError(format!(
                    "failed to parse quote_gas_payment result '{s}': {e}"
                ))
                .into()
            });
        }
        Err(HyperlaneAeternityError::ContractCallError(format!(
            "expected integer from quote_gas_payment(), got {result}"
        ))
        .into())
    }

    /// Pay for gas on the destination chain.
    ///
    /// Calls Sophia entrypoint: `pay_for_gas(message_id: bytes(32), dest_domain: int, gas_amount: int)`
    /// The contract uses Call.caller as refund address internally.
    pub async fn pay_for_gas(
        &self,
        message_id: H256,
        destination_domain: u32,
        gas_amount: U256,
        _refund_address: H256,
    ) -> ChainResult<TxOutcome> {
        let payment = self
            .quote_gas_payment(destination_domain, gas_amount)
            .await?;
        let payment_amount = payment.as_u64();

        let msg_id_hex = format!("#{}", hex::encode(message_id.as_bytes()));

        self.provider
            .send_contract_call(
                &self.contract_address,
                "pay_for_gas",
                &[
                    msg_id_hex,
                    destination_domain.to_string(),
                    gas_amount.as_u128().to_string(),
                ],
                &contracts::IGP_SOURCE,
                payment_amount,
                0,
            )
            .await
    }
}

impl HyperlaneContract for AeInterchainGasPaymaster {
    fn address(&self) -> H256 {
        self.address_h256
    }
}

impl HyperlaneChain for AeInterchainGasPaymaster {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(self.provider.clone())
    }
}

#[async_trait]
impl InterchainGasPaymaster for AeInterchainGasPaymaster {}

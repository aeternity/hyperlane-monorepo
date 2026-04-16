use async_trait::async_trait;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use hyperlane_core::{
    ChainResult, ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneProvider, InterchainGasPaymaster, TxOutcome, H256, U256,
};

use crate::{
    h256_to_contract_address, AeternityProvider, FateValue, HyperlaneAeternityError,
};

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
        let contract_address = h256_to_contract_address(locator.address)?;
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
                vec![
                    FateValue::Integer(BigInt::from(destination_domain)),
                    FateValue::Integer(BigInt::from(gas_amount.as_u128())),
                ],
            )
            .await?;

        match result {
            FateValue::Integer(n) => {
                let val = n.to_u128().ok_or_else(|| {
                    HyperlaneAeternityError::ContractCallError(
                        "quote_gas_payment overflow for u128".into(),
                    )
                })?;
                Ok(U256::from(val))
            }
            other => Err(HyperlaneAeternityError::ContractCallError(format!(
                "expected Integer from quote_gas_payment(), got {:?}",
                other
            ))
            .into()),
        }
    }

    /// Pay for gas on the destination chain.
    ///
    /// Calls Sophia entrypoint: `pay_for_gas(message_id: bytes(32), dest_domain: int, gas_amount: int, refund_address: address)`
    pub async fn pay_for_gas(
        &self,
        message_id: H256,
        destination_domain: u32,
        gas_amount: U256,
        refund_address: H256,
    ) -> ChainResult<TxOutcome> {
        let refund_addr = h256_to_contract_address(refund_address)?;

        let payment = self
            .quote_gas_payment(destination_domain, gas_amount)
            .await?;
        let payment_amount = payment.as_u64();

        self.provider
            .send_contract_call(
                &self.contract_address,
                "pay_for_gas",
                vec![
                    FateValue::Bytes(message_id.as_bytes().to_vec()),
                    FateValue::Integer(BigInt::from(destination_domain)),
                    FateValue::Integer(BigInt::from(gas_amount.as_u128())),
                    FateValue::Address(refund_addr),
                ],
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

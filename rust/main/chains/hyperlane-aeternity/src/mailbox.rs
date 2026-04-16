use async_trait::async_trait;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

use hyperlane_core::{
    ChainCommunicationError, ChainResult, ContractLocator, Encode, FixedPointNumber,
    HyperlaneChain, HyperlaneContract, HyperlaneDomain, HyperlaneMessage, HyperlaneProvider,
    Mailbox, Metadata, ReorgPeriod, TxCostEstimate, TxOutcome, H256, U256,
};

use crate::{
    contract_address_to_h256, h256_to_contract_address, AeternityProvider, FateValue,
    HyperlaneAeternityError,
};

/// Data required to send a contract call on Aeternity.
///
/// Serialized as JSON and returned by `process_calldata` / `delivered_calldata`
/// so the operation verifier can reconstruct the call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AeTxCalldata {
    /// Contract address in `ct_...` format
    pub contract_address: String,
    /// Sophia entrypoint to invoke
    pub function_name: String,
    /// Serialized arguments (JSON-encoded argument list)
    pub args: Vec<u8>,
}

/// Aeternity Mailbox
#[derive(Debug)]
pub struct AeMailbox {
    provider: AeternityProvider,
    contract_address: String,
    address_h256: H256,
    domain: HyperlaneDomain,
}

impl AeMailbox {
    /// Creates a new Aeternity Mailbox instance
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address)?;
        Ok(Self {
            domain: provider.domain().clone(),
            provider,
            contract_address,
            address_h256: locator.address,
        })
    }

    /// Build the calldata for a `process(metadata, message)` call.
    fn build_process_calldata(
        contract_address: &str,
        message: &HyperlaneMessage,
        metadata: &[u8],
    ) -> ChainResult<Vec<u8>> {
        let message_bytes = message.to_vec();
        let metadata_bytes = metadata.to_vec();

        let args = serde_json::to_vec(&(&metadata_bytes, &message_bytes))
            .map_err(ChainCommunicationError::from_other)?;

        let data = AeTxCalldata {
            contract_address: contract_address.to_string(),
            function_name: "process".into(),
            args,
        };
        serde_json::to_vec(&data).map_err(ChainCommunicationError::from_other)
    }

    /// Build the calldata for a `delivered(message_id)` check.
    fn build_delivered_calldata(
        contract_address: &str,
        message_id: H256,
    ) -> ChainResult<Option<Vec<u8>>> {
        let id_bytes = message_id.as_bytes().to_vec();
        let args = serde_json::to_vec(&id_bytes)
            .map_err(ChainCommunicationError::from_other)?;

        let data = AeTxCalldata {
            contract_address: contract_address.to_string(),
            function_name: "delivered".into(),
            args,
        };
        let json = serde_json::to_vec(&data).map_err(ChainCommunicationError::from_other)?;
        Ok(Some(json))
    }
}

impl HyperlaneContract for AeMailbox {
    fn address(&self) -> H256 {
        self.address_h256
    }
}

impl HyperlaneChain for AeMailbox {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(self.provider.clone())
    }
}

#[async_trait]
impl Mailbox for AeMailbox {
    /// Returns the number of dispatched messages (nonce).
    async fn count(&self, _reorg_period: &ReorgPeriod) -> ChainResult<u32> {
        let result = self
            .provider
            .call_contract(&self.contract_address, "nonce", vec![])
            .await?;

        match result {
            FateValue::Integer(n) => n
                .to_u32()
                .ok_or_else(|| {
                    HyperlaneAeternityError::ContractCallError(
                        "nonce value overflow for u32".into(),
                    )
                    .into()
                }),
            other => Err(HyperlaneAeternityError::ContractCallError(format!(
                "expected Integer from nonce(), got {:?}",
                other
            ))
            .into()),
        }
    }

    /// Check if a message has been delivered.
    async fn delivered(&self, id: H256) -> ChainResult<bool> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "delivered",
                vec![FateValue::Bytes(id.as_bytes().to_vec())],
            )
            .await?;

        match result {
            FateValue::Boolean(b) => Ok(b),
            other => Err(HyperlaneAeternityError::ContractCallError(format!(
                "expected Boolean from delivered(), got {:?}",
                other
            ))
            .into()),
        }
    }

    /// Fetch the current default interchain security module address.
    async fn default_ism(&self) -> ChainResult<H256> {
        let result = self
            .provider
            .call_contract(&self.contract_address, "default_ism", vec![])
            .await?;

        match result {
            FateValue::Address(addr) => contract_address_to_h256(&addr),
            FateValue::None => Err(HyperlaneAeternityError::ContractCallError(
                "no default ISM configured".into(),
            )
            .into()),
            other => Err(HyperlaneAeternityError::ContractCallError(format!(
                "expected Address from default_ism(), got {:?}",
                other
            ))
            .into()),
        }
    }

    /// Get the ISM address for a specific recipient.
    async fn recipient_ism(&self, recipient: H256) -> ChainResult<H256> {
        let recipient_addr = h256_to_contract_address(recipient)?;
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "recipient_ism",
                vec![FateValue::Address(recipient_addr)],
            )
            .await?;

        match result {
            FateValue::Address(addr) => contract_address_to_h256(&addr),
            FateValue::None => self.default_ism().await,
            other => Err(HyperlaneAeternityError::ContractCallError(format!(
                "expected Address from recipient_ism(), got {:?}",
                other
            ))
            .into()),
        }
    }

    /// Process (deliver) a message on the destination chain.
    async fn process(
        &self,
        message: &HyperlaneMessage,
        metadata: &Metadata,
        _tx_gas_limit: Option<U256>,
    ) -> ChainResult<TxOutcome> {
        let message_bytes = message.to_vec();
        let metadata_bytes = metadata.to_vec();

        self.provider
            .send_contract_call(
                &self.contract_address,
                "process",
                vec![
                    FateValue::Bytes(metadata_bytes),
                    FateValue::Bytes(message_bytes),
                ],
                0,
                0,
            )
            .await
    }

    /// Estimate transaction costs to process a message.
    async fn process_estimate_costs(
        &self,
        message: &HyperlaneMessage,
        metadata: &Metadata,
    ) -> ChainResult<TxCostEstimate> {
        let message_bytes = message.to_vec();
        let metadata_bytes = metadata.to_vec();

        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "process",
                vec![
                    FateValue::Bytes(metadata_bytes),
                    FateValue::Bytes(message_bytes),
                ],
            )
            .await;

        match result {
            Ok(_) => Ok(TxCostEstimate {
                gas_limit: U256::from(1_000_000u64),
                gas_price: FixedPointNumber::try_from("1000000000")?,
                l2_gas_limit: None,
            }),
            Err(e) => Err(e),
        }
    }

    /// Get the calldata for a `process` transaction.
    async fn process_calldata(
        &self,
        message: &HyperlaneMessage,
        metadata: &Metadata,
    ) -> ChainResult<Vec<u8>> {
        Self::build_process_calldata(&self.contract_address, message, metadata)
    }

    /// Get the calldata for a `delivered` check.
    fn delivered_calldata(&self, message_id: H256) -> ChainResult<Option<Vec<u8>>> {
        Self::build_delivered_calldata(&self.contract_address, message_id)
    }
}

#[cfg(test)]
mod tests {
    use hyperlane_core::{Encode, HyperlaneMessage, H256};

    use super::{AeMailbox, AeTxCalldata};

    const MAILBOX_ADDRESS: &str = "ct_2MBn4mFPieVPgvSR6ePTkjUhJUBRhEoJDxKX3Kcf6ZBcayZSr";

    #[test]
    fn test_process_calldata() {
        let message = HyperlaneMessage::default();
        let metadata = vec![20, 30, 40, 50];
        let calldata = AeMailbox::build_process_calldata(MAILBOX_ADDRESS, &message, &metadata)
            .expect("Failed to build process calldata");

        let actual: AeTxCalldata =
            serde_json::from_slice(&calldata).expect("Failed to parse json");

        assert_eq!(actual.contract_address, MAILBOX_ADDRESS);
        assert_eq!(actual.function_name, "process");
        assert!(!actual.args.is_empty());

        let (decoded_metadata, decoded_message): (Vec<u8>, Vec<u8>) =
            serde_json::from_slice(&actual.args).expect("Failed to decode args");
        assert_eq!(decoded_metadata, metadata);
        assert_eq!(decoded_message, message.to_vec());
    }

    #[test]
    fn test_delivered_calldata() {
        let message_id = H256::random();
        let calldata = AeMailbox::build_delivered_calldata(MAILBOX_ADDRESS, message_id)
            .expect("Failed to build delivered calldata")
            .expect("Delivered calldata is empty");

        let actual: AeTxCalldata =
            serde_json::from_slice(&calldata).expect("Failed to parse json");

        assert_eq!(actual.contract_address, MAILBOX_ADDRESS);
        assert_eq!(actual.function_name, "delivered");
        assert!(!actual.args.is_empty());

        let decoded_id: Vec<u8> =
            serde_json::from_slice(&actual.args).expect("Failed to decode args");
        assert_eq!(decoded_id, message_id.as_bytes().to_vec());
    }
}

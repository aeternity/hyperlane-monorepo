use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use hyperlane_core::{
    ChainCommunicationError, ChainResult, ContractLocator, Encode, FixedPointNumber,
    HyperlaneChain, HyperlaneContract, HyperlaneDomain, HyperlaneMessage, HyperlaneProvider,
    Mailbox, Metadata, ReorgPeriod, TxCostEstimate, TxOutcome, H256, U256,
};

use crate::{
    contracts, h256_to_contract_address, AeternityProvider, HyperlaneAeternityError,
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
        let contract_address = h256_to_contract_address(locator.address);
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
            .call_contract(
                &self.contract_address,
                "nonce",
                &[],
                &contracts::MAILBOX_SOURCE,
            )
            .await?;

        result
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| {
                HyperlaneAeternityError::ContractCallError(format!(
                    "expected integer from nonce(), got {result}"
                ))
                .into()
            })
    }

    /// Check if a message has been delivered.
    async fn delivered(&self, id: H256) -> ChainResult<bool> {
        let id_hex = format!("#{}", hex::encode(id.as_bytes()));
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "delivered",
                &[id_hex],
                &contracts::MAILBOX_SOURCE,
            )
            .await?;

        result.as_bool().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected bool from delivered(), got {result}"
            ))
            .into()
        })
    }

    /// Fetch the current default interchain security module address.
    async fn default_ism(&self) -> ChainResult<H256> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "default_ism",
                &[],
                &contracts::MAILBOX_SOURCE,
            )
            .await?;

        parse_option_contract_address(&result)
    }

    /// Get the ISM address for a specific recipient.
    async fn recipient_ism(&self, recipient: H256) -> ChainResult<H256> {
        let recipient_addr = crate::h256_to_account_address(recipient);
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "get_recipient_ism",
                &[recipient_addr],
                &contracts::MAILBOX_SOURCE,
            )
            .await;

        match result {
            Ok(val) => parse_option_contract_address(&val),
            Err(_) => self.default_ism().await,
        }
    }

    /// Process (deliver) a message on the destination chain.
    async fn process(
        &self,
        message: &HyperlaneMessage,
        metadata: &Metadata,
        _tx_gas_limit: Option<U256>,
    ) -> ChainResult<TxOutcome> {
        let message_hex = format!("#{}", hex::encode(message.to_vec()));
        let metadata_hex = format!("#{}", hex::encode(metadata.to_vec()));

        self.provider
            .send_contract_call(
                &self.contract_address,
                "process",
                &[metadata_hex, message_hex],
                &contracts::MAILBOX_SOURCE,
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
        let message_hex = format!("#{}", hex::encode(message.to_vec()));
        let metadata_hex = format!("#{}", hex::encode(metadata.to_vec()));

        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "process",
                &[metadata_hex, message_hex],
                &contracts::MAILBOX_SOURCE,
            )
            .await;

        match result {
            Ok(_) => Ok(TxCostEstimate {
                gas_limit: U256::from(1_000_000u64),
                gas_price: FixedPointNumber::try_from(U256::from(1_000_000_000u64))?,
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

/// Parse `option(contract_ref)` decoded by the compiler.
///
/// The compiler decodes `Some(ct_...)` as `{"Some": ["ct_..."]}` and `None` as `"None"`.
fn parse_option_contract_address(value: &serde_json::Value) -> ChainResult<H256> {
    if let Some(map) = value.as_object() {
        if let Some(args) = map.get("Some") {
            if let Some(addr_str) = args.as_array().and_then(|a| a.first()).and_then(|v| v.as_str())
            {
                return crate::ae_address_to_h256(addr_str);
            }
        }
    }
    Err(HyperlaneAeternityError::ContractCallError(format!(
        "expected Some(ct_...) for ISM address, got {value}"
    ))
    .into())
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

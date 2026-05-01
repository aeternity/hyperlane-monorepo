use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use hyperlane_core::{
    ChainCommunicationError, ChainResult, ContractLocator, Encode, FixedPointNumber,
    HyperlaneChain, HyperlaneContract, HyperlaneDomain, HyperlaneMessage, HyperlaneProvider,
    Mailbox, Metadata, ReorgPeriod, TxCostEstimate, TxOutcome, H256, U256,
};

use crate::{
    contracts, h256_to_account_address, h256_to_contract_address, AeternityProvider,
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
        let contract_address = h256_to_contract_address(locator.address);
        Ok(Self {
            domain: provider.domain().clone(),
            provider,
            contract_address,
            address_h256: locator.address,
        })
    }

    /// Query the maximum allowed message body size from the Mailbox contract.
    /// Returns 0 if no limit is configured.
    pub async fn max_message_body_bytes(&self) -> ChainResult<u32> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "get_max_message_body_bytes",
                &[],
                &contracts::MAILBOX_SOURCE,
            )
            .await?;

        result.as_u64().map(|n| n as u32).ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected integer from get_max_message_body_bytes(), got {result}"
            ))
            .into()
        })
    }

    /// Query the deployment block height of the Mailbox contract.
    /// Used to set the indexer scan start point and avoid empty blocks.
    pub async fn deployed_block(&self) -> ChainResult<u64> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "deployed_block",
                &[],
                &contracts::MAILBOX_SOURCE,
            )
            .await?;

        result.as_u64().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected integer from deployed_block(), got {result}"
            ))
            .into()
        })
    }

    /// Build the calldata for a `process(metadata, message, recipient_addr, body_recipient_addr)` call.
    fn build_process_calldata(
        contract_address: &str,
        message: &HyperlaneMessage,
        metadata: &[u8],
    ) -> ChainResult<Vec<u8>> {
        let message_bytes = message.to_vec();
        let metadata_bytes = metadata.to_vec();
        let recipient_addr = h256_to_account_address(message.recipient);
        let body_recipient_addr = body_recipient_from_message(message)?;

        let args = serde_json::to_vec(&(
            &metadata_bytes,
            &message_bytes,
            &recipient_addr,
            &body_recipient_addr,
        ))
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
        let args = serde_json::to_vec(&id_bytes).map_err(ChainCommunicationError::from_other)?;

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
            .await;

        match result {
            Ok(val) => {
                info!(decoded = %val, "default_ism() raw decoded value");
                parse_option_contract_address(&val)
            }
            Err(e) => {
                warn!(error = %e, "default_ism() call_contract failed");
                Err(e)
            }
        }
    }

    /// Get the ISM address for a specific recipient.
    async fn recipient_ism(&self, recipient: H256) -> ChainResult<H256> {
        let recipient_addr = crate::h256_to_account_address(recipient);
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "recipient_ism_for",
                &[recipient_addr.clone()],
                &contracts::MAILBOX_SOURCE,
            )
            .await;

        match &result {
            Ok(val) => {
                info!(recipient = %recipient_addr, decoded = %val, "recipient_ism_for raw decoded")
            }
            Err(e) => {
                warn!(recipient = %recipient_addr, error = %e, "recipient_ism_for call failed")
            }
        }

        match result {
            Ok(val) => match parse_option_contract_address(&val) {
                Ok(h) => Ok(h),
                Err(_) => {
                    info!("recipient_ism_for returned non-option value, trying direct parse");
                    if let Some(addr_str) = val.as_str() {
                        crate::ae_address_to_h256(addr_str)
                    } else {
                        self.default_ism().await
                    }
                }
            },
            Err(_) => {
                info!("recipient_ism_for errored, falling back to default_ism");
                self.default_ism().await
            }
        }
    }

    /// Process (deliver) a message on the destination chain.
    ///
    /// The Sophia Mailbox.process takes four args:
    ///   process(metadata, message, recipient_addr, body_recipient_addr)
    /// because the FATE VM has no Bytes.to_address conversion.
    async fn process(
        &self,
        message: &HyperlaneMessage,
        metadata: &Metadata,
        _tx_gas_limit: Option<U256>,
    ) -> ChainResult<TxOutcome> {
        // Pre-flight: reject messages with zero recipient
        if message.recipient == H256::zero() {
            return Err(HyperlaneAeternityError::ContractCallError(
                "message has zero recipient — rejected by Mailbox".into(),
            )
            .into());
        }

        // Pre-flight: body size limit check to avoid wasting gas
        let max_bytes = self.max_message_body_bytes().await.unwrap_or(0);
        if max_bytes > 0 && message.body.len() as u32 > max_bytes {
            return Err(HyperlaneAeternityError::ContractCallError(format!(
                "message body {} bytes exceeds max_message_body_bytes {}",
                message.body.len(),
                max_bytes
            ))
            .into());
        }

        let message_hex = format!("Bytes.to_any_size(#{})", hex::encode(message.to_vec()));
        let fate_metadata = convert_metadata_sigs_to_vrs(metadata);
        let metadata_hex = format!("Bytes.to_any_size(#{})", hex::encode(&fate_metadata));
        let recipient_addr = h256_to_account_address(message.recipient);
        let body_recipient_addr = body_recipient_from_message(message)?;

        self.provider
            .send_contract_call(
                &self.contract_address,
                "process",
                &[
                    metadata_hex,
                    message_hex,
                    recipient_addr,
                    body_recipient_addr,
                ],
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
        let message_hex = format!("Bytes.to_any_size(#{})", hex::encode(message.to_vec()));
        let fate_metadata = convert_metadata_sigs_to_vrs(metadata);
        let metadata_hex = format!("Bytes.to_any_size(#{})", hex::encode(&fate_metadata));
        let recipient_addr = h256_to_account_address(message.recipient);
        let body_recipient_addr = body_recipient_from_message(message)?;

        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "process",
                &[
                    metadata_hex,
                    message_hex,
                    recipient_addr,
                    body_recipient_addr,
                ],
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

/// Size of the metadata header before signatures begin.
/// [0:32] merkleTreeHook, [32:64] Merkle root, [64:68] tree index.
const METADATA_SIGS_OFFSET: usize = 68;

/// Convert ECDSA signatures in Hyperlane metadata from Ethereum byte order
/// (R‖S‖V, each 65 bytes) to the byte order expected by the FATE VM
/// (V‖R‖S).  The first 68 bytes (hook address, root, index) are unchanged.
fn convert_metadata_sigs_to_vrs(metadata: &[u8]) -> Vec<u8> {
    if metadata.len() <= METADATA_SIGS_OFFSET {
        return metadata.to_vec();
    }

    let mut out = metadata[..METADATA_SIGS_OFFSET].to_vec();
    let sigs = &metadata[METADATA_SIGS_OFFSET..];

    for chunk in sigs.chunks(65) {
        if chunk.len() == 65 {
            // Ethereum: R(32) || S(32) || V(1)  →  FATE: V(1) || R(32) || S(32)
            out.push(chunk[64]); // V
            out.extend_from_slice(&chunk[..32]); // R
            out.extend_from_slice(&chunk[32..64]); // S
        } else {
            out.extend_from_slice(chunk);
        }
    }

    out
}

/// Extract the body recipient address from a Hyperlane message.
///
/// For warp route messages: the first 32 bytes of the body contain the recipient H256.
/// For ICA/ICQ and other middleware messages: body may be shorter than 32 bytes or use
/// a different encoding — in these cases, message.recipient (the router contract)
/// serves as the body_recipient for the Mailbox process() call.
fn body_recipient_from_message(message: &HyperlaneMessage) -> ChainResult<String> {
    if message.body.len() >= 32 {
        let mut recipient_bytes = [0u8; 32];
        recipient_bytes.copy_from_slice(&message.body[..32]);
        let recipient_h256 = H256::from(recipient_bytes);

        // Zero recipient in body = fall back to message.recipient (router contract)
        if recipient_h256 == H256::zero() {
            return Ok(h256_to_account_address(message.recipient));
        }
        Ok(h256_to_account_address(recipient_h256))
    } else {
        // Short body (ICA/ICQ messages) — use the router contract as body recipient
        Ok(h256_to_account_address(message.recipient))
    }
}

/// Parse `option(contract_ref)` decoded by the compiler.
///
/// The compiler decodes `Some(ct_...)` as `{"Some": ["ct_..."]}` and `None` as `"None"`.
fn parse_option_contract_address(value: &serde_json::Value) -> ChainResult<H256> {
    if let Some(map) = value.as_object() {
        if let Some(args) = map.get("Some") {
            if let Some(addr_str) = args
                .as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
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
        let body_recipient = H256::repeat_byte(0xAB);
        let amount_bytes = [0u8; 32];
        let mut body = Vec::with_capacity(64);
        body.extend_from_slice(body_recipient.as_bytes());
        body.extend_from_slice(&amount_bytes);

        let message = HyperlaneMessage {
            body,
            ..Default::default()
        };
        let metadata = vec![20, 30, 40, 50];
        let calldata = AeMailbox::build_process_calldata(MAILBOX_ADDRESS, &message, &metadata)
            .expect("Failed to build process calldata");

        let actual: AeTxCalldata = serde_json::from_slice(&calldata).expect("Failed to parse json");

        assert_eq!(actual.contract_address, MAILBOX_ADDRESS);
        assert_eq!(actual.function_name, "process");
        assert!(!actual.args.is_empty());

        let (decoded_metadata, decoded_message, recipient_addr, body_recipient_addr): (
            Vec<u8>,
            Vec<u8>,
            String,
            String,
        ) = serde_json::from_slice(&actual.args).expect("Failed to decode args");
        assert_eq!(decoded_metadata, metadata);
        assert_eq!(decoded_message, message.to_vec());
        assert!(recipient_addr.starts_with("ak_"));
        assert!(body_recipient_addr.starts_with("ak_"));
    }

    #[test]
    fn test_convert_metadata_sigs_to_vrs() {
        use super::convert_metadata_sigs_to_vrs;

        // 68-byte header + one 65-byte sig in R(32)||S(32)||V(1) format
        let mut metadata = vec![0xAAu8; 68]; // header
        let r = [0x11u8; 32];
        let s = [0x22u8; 32];
        let v = 0x1Bu8; // 27
        metadata.extend_from_slice(&r);
        metadata.extend_from_slice(&s);
        metadata.push(v);
        assert_eq!(metadata.len(), 68 + 65);

        let converted = convert_metadata_sigs_to_vrs(&metadata);
        assert_eq!(converted.len(), metadata.len());
        assert_eq!(&converted[..68], &metadata[..68]); // header unchanged
        assert_eq!(converted[68], v); // V first
        assert_eq!(&converted[69..101], &r); // then R
        assert_eq!(&converted[101..133], &s); // then S
    }

    #[test]
    fn test_convert_metadata_sigs_two_sigs() {
        use super::convert_metadata_sigs_to_vrs;

        let mut metadata = vec![0u8; 68];
        for i in 0..2u8 {
            metadata.extend_from_slice(&[i + 1; 32]); // R
            metadata.extend_from_slice(&[i + 3; 32]); // S
            metadata.push(27 + i); // V
        }

        let converted = convert_metadata_sigs_to_vrs(&metadata);
        // First sig
        assert_eq!(converted[68], 27);
        assert_eq!(&converted[69..101], &[1u8; 32]);
        assert_eq!(&converted[101..133], &[3u8; 32]);
        // Second sig
        assert_eq!(converted[133], 28);
        assert_eq!(&converted[134..166], &[2u8; 32]);
        assert_eq!(&converted[166..198], &[4u8; 32]);
    }

    #[test]
    fn test_body_recipient_short_body_uses_message_recipient() {
        use super::body_recipient_from_message;
        let recipient = H256::repeat_byte(0xBB);
        let message = HyperlaneMessage {
            recipient,
            body: vec![0x01, 0x02, 0x03], // too short for warp route format
            ..Default::default()
        };
        let result = body_recipient_from_message(&message).unwrap();
        assert!(result.starts_with("ak_"));
    }

    #[test]
    fn test_body_recipient_zero_body_falls_back() {
        use super::body_recipient_from_message;
        let recipient = H256::repeat_byte(0xCC);
        let message = HyperlaneMessage {
            recipient,
            body: vec![0u8; 64], // first 32 bytes are zero
            ..Default::default()
        };
        let result = body_recipient_from_message(&message).unwrap();
        let expected = crate::h256_to_account_address(recipient);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_body_recipient_warp_route_extracts_from_body() {
        use super::body_recipient_from_message;
        let body_recipient = H256::repeat_byte(0xDD);
        let message_recipient = H256::repeat_byte(0xEE);
        let mut body = Vec::with_capacity(64);
        body.extend_from_slice(body_recipient.as_bytes());
        body.extend_from_slice(&[0u8; 32]);

        let message = HyperlaneMessage {
            recipient: message_recipient,
            body,
            ..Default::default()
        };
        let result = body_recipient_from_message(&message).unwrap();
        let expected = crate::h256_to_account_address(body_recipient);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_delivered_calldata() {
        let message_id = H256::random();
        let calldata = AeMailbox::build_delivered_calldata(MAILBOX_ADDRESS, message_id)
            .expect("Failed to build delivered calldata")
            .expect("Delivered calldata is empty");

        let actual: AeTxCalldata = serde_json::from_slice(&calldata).expect("Failed to parse json");

        assert_eq!(actual.contract_address, MAILBOX_ADDRESS);
        assert_eq!(actual.function_name, "delivered");
        assert!(!actual.args.is_empty());

        let decoded_id: Vec<u8> =
            serde_json::from_slice(&actual.args).expect("Failed to decode args");
        assert_eq!(decoded_id, message_id.as_bytes().to_vec());
    }
}

use std::io::Cursor;

use once_cell::sync::Lazy;

use hyperlane_core::{
    Decode, HyperlaneMessage, Indexed, InterchainGasPayment, LogMeta, MerkleTreeInsertion, H256,
    H512, U256,
};

use crate::utils::{blake2b_hex, decode_ae_hash};
use crate::types::contract_address_to_h256;

/// Middleware log entry used internally by event parsers.
///
/// This mirrors the fields from `crate::rpc::ContractLogEntry` with types
/// convenient for parsing (non-optional `event_hash`, `u64` heights).
#[derive(Debug, Clone)]
pub struct ContractLogEntry {
    pub contract_id: String,
    pub call_tx_hash: String,
    pub block_hash: String,
    pub height: u64,
    pub micro_index: u64,
    pub log_idx: u64,
    pub event_hash: String,
    pub args: Vec<String>,
    pub data: String,
}

impl From<&crate::rpc::ContractLogEntry> for ContractLogEntry {
    fn from(mdw: &crate::rpc::ContractLogEntry) -> Self {
        Self {
            contract_id: mdw.contract_id.clone(),
            call_tx_hash: mdw.call_tx_hash.clone(),
            block_hash: mdw.block_hash.clone(),
            height: mdw.height,
            micro_index: mdw.micro_index,
            log_idx: mdw.log_idx,
            event_hash: mdw.event_hash.clone().unwrap_or_default(),
            args: mdw.args.clone(),
            data: mdw.data.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Event hash constants (Blake2b-256 of Sophia event constructor names)
// ---------------------------------------------------------------------------

pub(crate) static DISPATCH_EVENT_HASH: Lazy<String> = Lazy::new(|| blake2b_hex("Dispatch"));
#[allow(dead_code)]
pub(crate) static DISPATCH_ID_EVENT_HASH: Lazy<String> = Lazy::new(|| blake2b_hex("DispatchId"));
#[allow(dead_code)]
pub(crate) static PROCESS_EVENT_HASH: Lazy<String> = Lazy::new(|| blake2b_hex("Process"));
pub(crate) static PROCESS_ID_EVENT_HASH: Lazy<String> = Lazy::new(|| blake2b_hex("ProcessId"));
pub(crate) static INSERTED_INTO_TREE_HASH: Lazy<String> =
    Lazy::new(|| blake2b_hex("InsertedIntoTree"));
pub(crate) static GAS_PAYMENT_HASH: Lazy<String> = Lazy::new(|| blake2b_hex("GasPayment"));

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum EventParseError {
    #[error("Invalid event data: {0}")]
    InvalidData(String),
    #[error("Missing field: {0}")]
    MissingField(String),
}

// ---------------------------------------------------------------------------
// LogMeta builder
// ---------------------------------------------------------------------------

/// Build a [`LogMeta`] from an AE middleware log entry.
pub fn build_log_meta(log: &ContractLogEntry) -> LogMeta {
    let address = contract_address_to_h256(&log.contract_id).unwrap_or_default();
    let block_hash = decode_ae_hash(&log.block_hash).unwrap_or_default();

    let mut tx_id = H512::zero();
    if let Ok(h) = decode_ae_hash(&log.call_tx_hash) {
        tx_id.0[..32].copy_from_slice(h.as_bytes());
    }

    LogMeta {
        address,
        block_number: log.height,
        block_hash,
        transaction_id: tx_id,
        transaction_index: log.micro_index,
        log_index: U256::from(log.log_idx),
    }
}

// ---------------------------------------------------------------------------
// Topic / data helpers
// ---------------------------------------------------------------------------

/// Parse a hex-encoded topic arg to 32 bytes (H256).
/// AE topics are big-endian integer hex strings.
fn parse_topic_h256(hex_str: &str) -> Result<H256, EventParseError> {
    let stripped = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(stripped)
        .map_err(|e| EventParseError::InvalidData(format!("hex decode topic: {e}")))?;
    if bytes.len() > 32 {
        return Err(EventParseError::InvalidData(format!(
            "topic too long: {} bytes",
            bytes.len()
        )));
    }
    let mut padded = [0u8; 32];
    padded[32 - bytes.len()..].copy_from_slice(&bytes);
    Ok(H256::from(padded))
}

/// Parse a hex-encoded topic arg to a u32.
fn parse_topic_u32(hex_str: &str) -> Result<u32, EventParseError> {
    let stripped = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    u64::from_str_radix(stripped, 16)
        .map(|v| v as u32)
        .map_err(|e| EventParseError::InvalidData(format!("parse u32 topic: {e}")))
}

/// Parse a hex-encoded topic arg to a U256.
fn parse_topic_u256(hex_str: &str) -> Result<U256, EventParseError> {
    let stripped = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(stripped)
        .map_err(|e| EventParseError::InvalidData(format!("hex decode U256: {e}")))?;
    Ok(U256::from_big_endian(&bytes))
}

/// Decode the non-indexed `data` field from an AE event.
///
/// The data is base64-encoded with a "cb_" prefix. Stripping the prefix
/// and decoding gives the raw FATE-serialized bytes.
fn decode_event_data(data: &str) -> Result<Vec<u8>, EventParseError> {
    let b64 = data
        .strip_prefix("cb_")
        .ok_or_else(|| EventParseError::InvalidData("data missing cb_ prefix".into()))?;

    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD
        .decode(b64)
        .map_err(|e| EventParseError::InvalidData(format!("base64 decode: {e}")))
}

// ---------------------------------------------------------------------------
// Parse functions
// ---------------------------------------------------------------------------

/// Parse a Dispatch event from a middleware log entry.
///
/// Sophia event layout:
///   indexed: sender (address), destination (int), recipient (bytes(32))
///   data:    FATE-encoded raw message bytes
pub fn parse_dispatch_event(
    log: &ContractLogEntry,
) -> Result<Option<(Indexed<HyperlaneMessage>, LogMeta)>, EventParseError> {
    if log.event_hash != *DISPATCH_EVENT_HASH {
        return Ok(None);
    }

    if log.args.len() < 3 {
        return Err(EventParseError::MissingField(
            "Dispatch requires at least 3 topic args".into(),
        ));
    }

    let message_bytes = decode_event_data(&log.data)?;

    let mut reader = Cursor::new(&message_bytes);
    let message = HyperlaneMessage::read_from(&mut reader).map_err(|e| {
        EventParseError::InvalidData(format!("failed to decode HyperlaneMessage: {e}"))
    })?;

    let sequence = message.nonce;
    let meta = build_log_meta(log);

    Ok(Some((
        Indexed::new(message).with_sequence(sequence),
        meta,
    )))
}

/// Parse a ProcessId (delivery) event from a middleware log entry.
///
/// Sophia event layout:
///   indexed: message_id (bytes(32))
///   data:    FATE-encoded sequence (int)
pub fn parse_delivery_event(
    log: &ContractLogEntry,
) -> Result<Option<(Indexed<H256>, LogMeta)>, EventParseError> {
    if log.event_hash != *PROCESS_ID_EVENT_HASH {
        return Ok(None);
    }

    if log.args.is_empty() {
        return Err(EventParseError::MissingField(
            "ProcessId requires at least 1 topic arg".into(),
        ));
    }

    let message_id = parse_topic_h256(&log.args[0])?;

    let sequence = if log.args.len() > 1 {
        parse_topic_u32(&log.args[1]).unwrap_or(0)
    } else {
        0
    };

    let meta = build_log_meta(log);
    Ok(Some((
        Indexed::new(message_id).with_sequence(sequence),
        meta,
    )))
}

/// Parse an InsertedIntoTree event from a middleware log entry.
///
/// Sophia event layout:
///   indexed: message_id (bytes(32)), leaf_index (int)
pub fn parse_merkle_insertion(
    log: &ContractLogEntry,
) -> Result<Option<(Indexed<MerkleTreeInsertion>, LogMeta)>, EventParseError> {
    if log.event_hash != *INSERTED_INTO_TREE_HASH {
        return Ok(None);
    }

    if log.args.len() < 2 {
        return Err(EventParseError::MissingField(
            "InsertedIntoTree requires at least 2 topic args".into(),
        ));
    }

    let message_id = parse_topic_h256(&log.args[0])?;
    let leaf_index = parse_topic_u32(&log.args[1])?;

    let insertion = MerkleTreeInsertion::new(leaf_index, message_id);
    let meta = build_log_meta(log);

    Ok(Some((
        Indexed::new(insertion).with_sequence(leaf_index),
        meta,
    )))
}

/// Parse a GasPayment event from a middleware log entry.
///
/// Sophia event layout:
///   indexed: message_id (bytes(32)), destination (int), gas_amount (int), payment (int)
pub fn parse_gas_payment(
    log: &ContractLogEntry,
) -> Result<Option<(Indexed<InterchainGasPayment>, LogMeta)>, EventParseError> {
    if log.event_hash != *GAS_PAYMENT_HASH {
        return Ok(None);
    }

    if log.args.len() < 4 {
        return Err(EventParseError::MissingField(
            "GasPayment requires at least 4 topic args".into(),
        ));
    }

    let message_id = parse_topic_h256(&log.args[0])?;
    let destination = parse_topic_u32(&log.args[1])?;
    let gas_amount = parse_topic_u256(&log.args[2])?;
    let payment = parse_topic_u256(&log.args[3])?;

    let sequence = if log.args.len() > 4 {
        parse_topic_u32(&log.args[4]).unwrap_or(0)
    } else {
        0
    };

    let gas_payment = InterchainGasPayment {
        message_id,
        destination,
        payment,
        gas_amount,
    };

    let meta = build_log_meta(log);

    Ok(Some((
        Indexed::new(gas_payment).with_sequence(sequence),
        meta,
    )))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_log(event_hash: &str, args: Vec<&str>, data: &str) -> ContractLogEntry {
        ContractLogEntry {
            contract_id: "ct_2swhLkgBPeeADxVTAby2CaivPq43LDJzWGKfSstbn1epwMuQzv".into(),
            call_tx_hash: "th_2swhLkgBPeeADxVTAby2CaivPq43LDJzWGKfSstbn1epwMuQzv".into(),
            block_hash: "mh_2swhLkgBPeeADxVTAby2CaivPq43LDJzWGKfSstbn1epwMuQzv".into(),
            height: 100u64,
            micro_index: 0u64,
            log_idx: 0u64,
            event_hash: event_hash.into(),
            args: args.into_iter().map(String::from).collect(),
            data: data.into(),
        }
    }

    #[test]
    fn test_blake2b_event_hashes_are_deterministic() {
        let h1 = blake2b_hex("Dispatch");
        let h2 = blake2b_hex("Dispatch");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_different_events_have_different_hashes() {
        assert_ne!(*DISPATCH_EVENT_HASH, *PROCESS_EVENT_HASH);
        assert_ne!(*DISPATCH_EVENT_HASH, *INSERTED_INTO_TREE_HASH);
        assert_ne!(*PROCESS_ID_EVENT_HASH, *GAS_PAYMENT_HASH);
    }

    #[test]
    fn test_parse_dispatch_wrong_hash_returns_none() {
        let log = make_log("0xdeadbeef", vec![], "cb_");
        let result = parse_dispatch_event(&log).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_delivery_wrong_hash_returns_none() {
        let log = make_log("0xdeadbeef", vec![], "");
        let result = parse_delivery_event(&log).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_merkle_insertion_wrong_hash_returns_none() {
        let log = make_log("0xdeadbeef", vec![], "");
        let result = parse_merkle_insertion(&log).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_gas_payment_wrong_hash_returns_none() {
        let log = make_log("0xdeadbeef", vec![], "");
        let result = parse_gas_payment(&log).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_topic_h256_padded() {
        let h = parse_topic_h256("0x01").unwrap();
        let mut expected = [0u8; 32];
        expected[31] = 1;
        assert_eq!(h, H256::from(expected));
    }

    #[test]
    fn test_parse_topic_u32() {
        assert_eq!(parse_topic_u32("0x2a").unwrap(), 42);
        assert_eq!(parse_topic_u32("ff").unwrap(), 255);
    }

    #[test]
    fn test_build_log_meta_fields() {
        let log = make_log("", vec![], "");
        let meta = build_log_meta(&log);
        assert_eq!(meta.block_number, 100);
        assert_eq!(meta.transaction_index, 0);
        assert_eq!(meta.log_index, U256::from(0));
    }

    #[test]
    fn test_parse_merkle_insertion_success() {
        let msg_id = "0x".to_owned() + &"ab".repeat(32);
        let leaf_idx = "0x05";
        let log = make_log(&INSERTED_INTO_TREE_HASH, vec![&msg_id, leaf_idx], "");
        let result = parse_merkle_insertion(&log).unwrap();
        assert!(result.is_some());
        let (indexed, _meta) = result.unwrap();
        assert_eq!(indexed.sequence, Some(5));
    }

    #[test]
    fn test_parse_delivery_success() {
        let msg_id = "0x".to_owned() + &"cd".repeat(32);
        let seq = "0x03";
        let log = make_log(&PROCESS_ID_EVENT_HASH, vec![&msg_id, seq], "");
        let result = parse_delivery_event(&log).unwrap();
        assert!(result.is_some());
        let (indexed, _meta) = result.unwrap();
        assert_eq!(indexed.sequence, Some(3));
    }

    #[test]
    fn test_parse_gas_payment_success() {
        let msg_id = "0x".to_owned() + &"ee".repeat(32);
        let dest = "0x01";
        let gas = "0x64";
        let payment = "0xc8";
        let seq = "0x07";
        let log = make_log(
            &GAS_PAYMENT_HASH,
            vec![&msg_id, dest, gas, payment, seq],
            "",
        );
        let result = parse_gas_payment(&log).unwrap();
        assert!(result.is_some());
        let (indexed, _meta) = result.unwrap();
        assert_eq!(indexed.sequence, Some(7));
    }
}

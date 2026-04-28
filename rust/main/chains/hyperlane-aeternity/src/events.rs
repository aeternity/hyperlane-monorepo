use std::io::Cursor;

use once_cell::sync::Lazy;

use hyperlane_core::{
    Decode, HyperlaneMessage, Indexed, InterchainGasPayment, LogMeta, MerkleTreeInsertion, H256,
    H512, U256,
};

use crate::types::contract_address_to_h256;
use crate::utils::{base32hex_to_hex, blake2b_hex, decode_ae_hash};

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
            event_hash: mdw
                .event_hash
                .as_deref()
                .map(|h| {
                    // Middleware returns event hashes in base32hex encoding;
                    // normalize to hex so comparisons with blake2b_hex() work.
                    base32hex_to_hex(h).unwrap_or_else(|| h.to_string())
                })
                .unwrap_or_default(),
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

/// Parse a topic arg to 32 bytes (H256).
///
/// The middleware returns topic args as decimal integer strings,
/// while some test fixtures use 0x-prefixed hex. Handle both.
fn parse_topic_h256(s: &str) -> Result<H256, EventParseError> {
    if let Some(hex_part) = s.strip_prefix("0x") {
        let bytes = hex::decode(hex_part)
            .map_err(|e| EventParseError::InvalidData(format!("hex decode topic: {e}")))?;
        if bytes.len() > 32 {
            return Err(EventParseError::InvalidData(format!(
                "topic too long: {} bytes",
                bytes.len()
            )));
        }
        let mut padded = [0u8; 32];
        padded[32 - bytes.len()..].copy_from_slice(&bytes);
        return Ok(H256::from(padded));
    }
    // Decimal integer string from middleware
    let n = num_bigint::BigUint::parse_bytes(s.as_bytes(), 10)
        .ok_or_else(|| EventParseError::InvalidData(format!("invalid decimal topic: {s}")))?;
    let be_bytes = n.to_bytes_be();
    if be_bytes.len() > 32 {
        return Err(EventParseError::InvalidData(format!(
            "topic too long: {} bytes",
            be_bytes.len()
        )));
    }
    let mut padded = [0u8; 32];
    padded[32 - be_bytes.len()..].copy_from_slice(&be_bytes);
    Ok(H256::from(padded))
}

/// Parse a topic arg to a u32.
fn parse_topic_u32(s: &str) -> Result<u32, EventParseError> {
    if let Some(hex_part) = s.strip_prefix("0x") {
        return u64::from_str_radix(hex_part, 16)
            .map(|v| v as u32)
            .map_err(|e| EventParseError::InvalidData(format!("parse u32 topic: {e}")));
    }
    s.parse::<u64>()
        .map(|v| v as u32)
        .map_err(|e| EventParseError::InvalidData(format!("parse u32 topic: {e}")))
}

/// Parse a topic arg to a U256.
fn parse_topic_u256(s: &str) -> Result<U256, EventParseError> {
    if let Some(hex_part) = s.strip_prefix("0x") {
        let bytes = hex::decode(hex_part)
            .map_err(|e| EventParseError::InvalidData(format!("hex decode U256: {e}")))?;
        return Ok(U256::from_big_endian(&bytes));
    }
    let n = num_bigint::BigUint::parse_bytes(s.as_bytes(), 10)
        .ok_or_else(|| EventParseError::InvalidData(format!("invalid decimal U256: {s}")))?;
    let be_bytes = n.to_bytes_be();
    Ok(U256::from_big_endian(&be_bytes))
}

/// Decode the non-indexed `data` field from an AE event.
///
/// The data is base64-encoded with a "cb_" prefix. Stripping the prefix
/// and decoding gives the raw FATE-serialized bytes.
fn decode_event_data(data: &str) -> Result<Vec<u8>, EventParseError> {
    // The AE middleware returns event data as plain base64.
    // The AE compiler/node uses "cb_" prefixed base64 (with a 4-byte checksum).
    // Handle both formats gracefully.
    let b64 = data.strip_prefix("cb_").unwrap_or(data);

    use base64::{engine::general_purpose::STANDARD, Engine};
    let decoded = STANDARD
        .decode(b64)
        .map_err(|e| EventParseError::InvalidData(format!("base64 decode: {e}")))?;

    // If decoded from cb_ format, strip the 4-byte SHA-256 checksum suffix
    if data.starts_with("cb_") && decoded.len() > 4 {
        Ok(decoded[..decoded.len() - 4].to_vec())
    } else {
        Ok(decoded)
    }
}

/// Parse a non-indexed event `data` field containing a decimal integer string
/// into a U256.
///
/// The AE middleware returns `string` event fields as plain text (e.g. `"200000"`),
/// while the node API uses base64 with an optional `cb_` prefix.  We try plain
/// decimal first and fall back to base64 decoding.
fn parse_data_string_as_u256(data: &str) -> Result<U256, EventParseError> {
    if data.is_empty() {
        return Err(EventParseError::MissingField(
            "event data is empty, expected gas_amount string".into(),
        ));
    }

    let trimmed = data.trim();

    // Fast path: middleware returns the string value as plain decimal text
    if let Some(n) = num_bigint::BigUint::parse_bytes(trimmed.as_bytes(), 10) {
        return Ok(U256::from_big_endian(&n.to_bytes_be()));
    }

    // Slow path: base64-encoded (node API / cb_ format)
    let raw = decode_event_data(data)?;
    let s = String::from_utf8(raw)
        .map_err(|e| EventParseError::InvalidData(format!("data not valid UTF-8: {e}")))?;
    let s_trimmed = s.trim();
    let n = num_bigint::BigUint::parse_bytes(s_trimmed.as_bytes(), 10).ok_or_else(|| {
        EventParseError::InvalidData(format!("invalid decimal in event data: {s_trimmed}"))
    })?;
    Ok(U256::from_big_endian(&n.to_bytes_be()))
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

    Ok(Some((Indexed::new(message).with_sequence(sequence), meta)))
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
///   indexed: message_id (bytes(32)), destination (int), payment (int)
///   data:    gas_amount as decimal string (non-indexed)
///
/// `gas_amount` is emitted as a `string` (via `Int.to_str`) to stay within
/// Sophia's 3-indexed-field limit.  The raw UTF-8 decimal lives in `log.data`.
pub fn parse_gas_payment(
    log: &ContractLogEntry,
) -> Result<Option<(Indexed<InterchainGasPayment>, LogMeta)>, EventParseError> {
    if log.event_hash != *GAS_PAYMENT_HASH {
        return Ok(None);
    }

    if log.args.len() < 3 {
        return Err(EventParseError::MissingField(
            "GasPayment requires at least 3 topic args".into(),
        ));
    }

    let message_id = parse_topic_h256(&log.args[0])?;
    let destination = parse_topic_u32(&log.args[1])?;
    let payment = parse_topic_u256(&log.args[2])?;
    let gas_amount = parse_data_string_as_u256(&log.data)?;

    let sequence = if log.args.len() > 3 {
        parse_topic_u32(&log.args[3]).unwrap_or(0)
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

    /// Base64-encode a plain string for use as event data in test fixtures.
    fn encode_test_data(s: &str) -> String {
        use base64::{engine::general_purpose::STANDARD, Engine};
        STANDARD.encode(s.as_bytes())
    }

    #[test]
    fn test_parse_gas_payment_success() {
        let msg_id = "0x".to_owned() + &"ee".repeat(32);
        let dest = "0x01";
        let payment = "0xc8";
        let seq = "0x07";
        let gas_data = encode_test_data("100");
        let log = make_log(
            &GAS_PAYMENT_HASH,
            vec![&msg_id, dest, payment, seq],
            &gas_data,
        );
        let result = parse_gas_payment(&log).unwrap();
        assert!(result.is_some());
        let (indexed, _meta) = result.unwrap();
        assert_eq!(indexed.inner().gas_amount, U256::from(100));
        assert_eq!(indexed.inner().payment, U256::from(200));
        assert_eq!(indexed.sequence, Some(7));
    }

    #[test]
    fn test_parse_gas_payment_no_data() {
        let msg_id = "0x".to_owned() + &"ee".repeat(32);
        let dest = "0x01";
        let payment = "0xc8";
        let log = make_log(&GAS_PAYMENT_HASH, vec![&msg_id, dest, payment], "");
        let result = parse_gas_payment(&log);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_gas_payment_large_gas_amount() {
        let msg_id = "0x".to_owned() + &"aa".repeat(32);
        let dest = "0x01";
        let payment = "0x01";
        let gas_data = encode_test_data("1000000000000000000");
        let log = make_log(&GAS_PAYMENT_HASH, vec![&msg_id, dest, payment], &gas_data);
        let result = parse_gas_payment(&log).unwrap();
        assert!(result.is_some());
        let (indexed, _meta) = result.unwrap();
        assert_eq!(
            indexed.inner().gas_amount,
            U256::from(1_000_000_000_000_000_000u64)
        );
    }

    #[test]
    fn test_parse_gas_payment_zero_gas() {
        let msg_id = "0x".to_owned() + &"bb".repeat(32);
        let dest = "0x01";
        let payment = "0x01";
        let gas_data = encode_test_data("0");
        let log = make_log(&GAS_PAYMENT_HASH, vec![&msg_id, dest, payment], &gas_data);
        let result = parse_gas_payment(&log).unwrap();
        assert!(result.is_some());
        let (indexed, _meta) = result.unwrap();
        assert_eq!(indexed.inner().gas_amount, U256::zero());
    }

    #[test]
    fn test_parse_data_string_as_u256_plain_decimal() {
        assert_eq!(
            parse_data_string_as_u256("200000").unwrap(),
            U256::from(200_000)
        );
    }

    #[test]
    fn test_parse_data_string_as_u256_plain_base64() {
        let data = encode_test_data("42");
        assert_eq!(parse_data_string_as_u256(&data).unwrap(), U256::from(42));
    }

    #[test]
    fn test_parse_data_string_as_u256_cb_prefix() {
        use base64::{engine::general_purpose::STANDARD, Engine};
        use sha2::{Digest, Sha256};
        let payload = b"12345";
        let mut buf = payload.to_vec();
        let hash = Sha256::digest(&buf);
        buf.extend_from_slice(&hash[..4]);
        let data = format!("cb_{}", STANDARD.encode(&buf));
        assert_eq!(parse_data_string_as_u256(&data).unwrap(), U256::from(12345));
    }

    #[test]
    fn test_parse_data_string_as_u256_empty() {
        assert!(parse_data_string_as_u256("").is_err());
    }

    #[test]
    fn test_parse_data_string_as_u256_non_numeric() {
        let data = encode_test_data("not_a_number");
        assert!(parse_data_string_as_u256(&data).is_err());
    }

    #[test]
    fn test_parse_gas_payment_plain_decimal_data() {
        let msg_id = "0x".to_owned() + &"ee".repeat(32);
        let dest = "11155111";
        let payment = "2857551541958855915";
        let log = make_log(&GAS_PAYMENT_HASH, vec![&msg_id, dest, payment], "200000");
        let result = parse_gas_payment(&log).unwrap();
        assert!(result.is_some());
        let (indexed, _meta) = result.unwrap();
        assert_eq!(indexed.inner().gas_amount, U256::from(200_000));
        assert_eq!(
            indexed.inner().payment,
            U256::from(2_857_551_541_958_855_915u64)
        );
        assert_eq!(indexed.inner().destination, 11_155_111);
    }

    #[test]
    fn test_parse_gas_payment_without_sequence() {
        let msg_id = "0x".to_owned() + &"cc".repeat(32);
        let dest = "0x05";
        let payment = "0x0a";
        let gas_data = encode_test_data("500");
        let log = make_log(&GAS_PAYMENT_HASH, vec![&msg_id, dest, payment], &gas_data);
        let result = parse_gas_payment(&log).unwrap();
        assert!(result.is_some());
        let (indexed, _meta) = result.unwrap();
        assert_eq!(indexed.inner().gas_amount, U256::from(500));
        assert_eq!(indexed.inner().payment, U256::from(10));
        assert_eq!(indexed.inner().destination, 5);
        assert_eq!(indexed.sequence, Some(0));
    }
}

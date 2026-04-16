use blake2::{digest::consts::U32, Blake2b, Digest};
use sha2::Sha256;

use hyperlane_core::{ChainResult, H256};

use crate::HyperlaneAeternityError;

/// Type alias for Blake2b with 256-bit output.
type Blake2b256 = Blake2b<U32>;

/// Compute a Blake2b-256 hash of `input`.
pub fn blake2b_256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Blake2b256::new();
    hasher.update(input);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Compute the hex-encoded Blake2b-256 hash of a UTF-8 string.
pub fn blake2b_hex(input: &str) -> String {
    hex::encode(blake2b_256(input.as_bytes()))
}

/// Decode an Aeternity prefixed hash (e.g. `th_...`, `kh_...`, `mh_...`) into an [`H256`].
///
/// The payload is base58check-decoded (same scheme as addresses).
pub fn decode_ae_hash(hash: &str) -> ChainResult<H256> {
    let (_prefix, body) = hash
        .split_once('_')
        .ok_or_else(|| HyperlaneAeternityError::AddressError("missing prefix separator".into()))?;

    let decoded = bs58::decode(body)
        .into_vec()
        .map_err(|e| HyperlaneAeternityError::AddressError(format!("base58 decode: {e}")))?;

    if decoded.len() < 4 {
        return Err(
            HyperlaneAeternityError::AddressError("decoded hash too short".into()).into(),
        );
    }

    let (payload, checksum) = decoded.split_at(decoded.len() - 4);
    let expected = &Sha256::digest(Sha256::digest(payload))[..4];
    if checksum != expected {
        return Err(HyperlaneAeternityError::AddressError("hash checksum mismatch".into()).into());
    }

    if payload.len() != 32 {
        return Err(HyperlaneAeternityError::AddressError(format!(
            "expected 32-byte hash payload, got {}",
            payload.len()
        ))
        .into());
    }

    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(payload);
    Ok(H256::from(bytes))
}

/// Encode raw bytes into an Aeternity prefixed hash string (e.g. `th_...`).
pub fn encode_ae_hash(bytes: &[u8], prefix: &str) -> ChainResult<String> {
    if bytes.len() != 32 {
        return Err(HyperlaneAeternityError::AddressError(format!(
            "expected 32 bytes for hash, got {}",
            bytes.len()
        ))
        .into());
    }
    let checksum = &Sha256::digest(Sha256::digest(bytes))[..4];
    let mut buf = Vec::with_capacity(36);
    buf.extend_from_slice(bytes);
    buf.extend_from_slice(checksum);
    Ok(format!("{}_{}", prefix, bs58::encode(buf).into_string()))
}

/// Convert an AE node timestamp (milliseconds since epoch) to seconds.
pub fn ae_timestamp_to_seconds(timestamp_ms: u64) -> u64 {
    timestamp_ms / 1000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake2b_256_known_vector() {
        let hash = blake2b_256(b"");
        assert_eq!(hash.len(), 32);
        let hex_str = hex::encode(hash);
        // Blake2b-256 of empty input
        assert_eq!(
            hex_str,
            "0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8"
        );
    }

    #[test]
    fn test_blake2b_hex() {
        let result = blake2b_hex("hello");
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_hash_roundtrip() {
        let original = H256::from([0xABu8; 32]);
        let encoded = encode_ae_hash(original.as_bytes(), "th").unwrap();
        assert!(encoded.starts_with("th_"));
        let decoded = decode_ae_hash(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_decode_ae_hash_invalid() {
        let result = decode_ae_hash("xx_invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_ae_timestamp_to_seconds() {
        assert_eq!(ae_timestamp_to_seconds(1_700_000_000_000), 1_700_000_000);
        assert_eq!(ae_timestamp_to_seconds(0), 0);
    }
}

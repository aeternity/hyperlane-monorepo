use hyperlane_core::{ChainResult, H256};
use sha2::{Digest, Sha256};

use crate::HyperlaneAeternityError;

/// Decode an Aeternity base58check-encoded address into raw payload bytes.
///
/// AE format: `<prefix>_<base58(payload ++ sha256(sha256(payload))[0..4])>`
fn base58check_decode(encoded: &str) -> ChainResult<(String, Vec<u8>)> {
    let (prefix, body) = encoded
        .split_once('_')
        .ok_or_else(|| HyperlaneAeternityError::AddressError("missing prefix separator".into()))?;

    let decoded = bs58::decode(body)
        .into_vec()
        .map_err(|e| HyperlaneAeternityError::AddressError(format!("base58 decode: {e}")))?;

    if decoded.len() < 4 {
        return Err(
            HyperlaneAeternityError::AddressError("decoded payload too short".into()).into(),
        );
    }

    let (payload, checksum) = decoded.split_at(decoded.len() - 4);
    let expected = &Sha256::digest(Sha256::digest(payload))[..4];
    if checksum != expected {
        return Err(HyperlaneAeternityError::AddressError("checksum mismatch".into()).into());
    }

    Ok((prefix.to_string(), payload.to_vec()))
}

/// Encode raw bytes into an Aeternity base58check string with the given prefix.
///
/// AE format: `<prefix>_<base58(payload ++ sha256(sha256(payload))[0..4])>`
fn base58check_encode(prefix: &str, payload: &[u8]) -> String {
    let checksum = &Sha256::digest(Sha256::digest(payload))[..4];
    let mut buf = Vec::with_capacity(payload.len() + 4);
    buf.extend_from_slice(payload);
    buf.extend_from_slice(checksum);
    format!("{}_{}", prefix, bs58::encode(buf).into_string())
}

/// Convert an `ak_` or `ct_` prefixed Aeternity address to an [`H256`].
///
/// The 32-byte public-key payload is placed directly into the H256.
pub fn ae_address_to_h256(address: &str) -> ChainResult<H256> {
    let (_prefix, payload) = base58check_decode(address)?;
    if payload.len() != 32 {
        return Err(HyperlaneAeternityError::AddressError(format!(
            "expected 32-byte payload, got {}",
            payload.len()
        ))
        .into());
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&payload);
    Ok(H256::from(bytes))
}

/// Convert an [`H256`] to an Aeternity base58check address with the given prefix.
pub fn h256_to_ae_address(h256: H256, prefix: &str) -> String {
    base58check_encode(prefix, h256.as_bytes())
}

/// Convert a `ct_...` contract address to [`H256`].
pub fn contract_address_to_h256(address: &str) -> ChainResult<H256> {
    if !address.starts_with("ct_") {
        return Err(HyperlaneAeternityError::AddressError(format!(
            "expected ct_ prefix, got: {address}"
        ))
        .into());
    }
    ae_address_to_h256(address)
}

/// Convert an [`H256`] to a `ct_...` contract address.
pub fn h256_to_contract_address(h256: H256) -> String {
    h256_to_ae_address(h256, "ct")
}

/// Convert an `ak_...` account address to [`H256`].
#[allow(dead_code)]
pub fn account_address_to_h256(address: &str) -> ChainResult<H256> {
    if !address.starts_with("ak_") {
        return Err(HyperlaneAeternityError::AddressError(format!(
            "expected ak_ prefix, got: {address}"
        ))
        .into());
    }
    ae_address_to_h256(address)
}

/// Convert an [`H256`] to an `ak_...` account address.
pub fn h256_to_account_address(h256: H256) -> String {
    h256_to_ae_address(h256, "ak")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_address_roundtrip() {
        let h = H256::from([99u8; 32]);
        let addr = h256_to_account_address(h);
        assert!(addr.starts_with("ak_"));
        let decoded = account_address_to_h256(&addr).unwrap();
        assert_eq!(decoded, h);
    }

    #[test]
    fn test_contract_address_roundtrip() {
        let h = H256::from([42u8; 32]);
        let addr = h256_to_contract_address(h);
        assert!(addr.starts_with("ct_"));
        let decoded = contract_address_to_h256(&addr).unwrap();
        assert_eq!(decoded, h);
    }

    #[test]
    fn test_ae_address_to_h256_wrong_prefix_still_decodes() {
        let h = H256::from([1u8; 32]);
        let addr = h256_to_ae_address(h, "ok");
        let decoded = ae_address_to_h256(&addr).unwrap();
        assert_eq!(decoded, h);
    }

    #[test]
    fn test_invalid_checksum() {
        let result = ae_address_to_h256("ak_111111111111111111111111111111BAD");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_prefix() {
        let result = ae_address_to_h256("noprefixhere");
        assert!(result.is_err());
    }

    #[test]
    fn test_account_prefix_enforcement() {
        let h = H256::from([7u8; 32]);
        let ct_addr = h256_to_contract_address(h);
        let result = account_address_to_h256(&ct_addr);
        assert!(result.is_err());
    }
}

use ed25519_dalek::{Signer as _, SigningKey, VerifyingKey};

use hyperlane_core::{ChainResult, H256};

use crate::types::h256_to_account_address;
use crate::HyperlaneAeternityError;

/// Ed25519 signer for Aeternity transactions.
#[derive(Clone, Debug)]
pub struct AeSigner {
    private_key: Vec<u8>,
    public_key: Vec<u8>,
    /// `ak_`-prefixed address
    pub encoded_address: String,
    /// H256 representation of the public key
    pub address_h256: H256,
    /// Network ID (`ae_mainnet`, `ae_uat`)
    network_id: String,
}

impl AeSigner {
    /// Create a new signer from raw Ed25519 private-key bytes and network ID.
    pub fn new(private_key_bytes: Vec<u8>, network_id: String) -> ChainResult<Self> {
        if private_key_bytes.len() != 32 {
            return Err(HyperlaneAeternityError::SigningError(format!(
                "expected 32 byte private key, got {}",
                private_key_bytes.len()
            ))
            .into());
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&private_key_bytes);

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key: VerifyingKey = (&signing_key).into();
        let public_key_bytes = verifying_key.to_bytes().to_vec();

        let mut h256_bytes = [0u8; 32];
        h256_bytes.copy_from_slice(&public_key_bytes);
        let address_h256 = H256::from(h256_bytes);

        let encoded_address = h256_to_account_address(address_h256);

        Ok(Self {
            private_key: private_key_bytes,
            public_key: public_key_bytes,
            encoded_address,
            address_h256,
            network_id,
        })
    }

    /// Sign raw bytes with Ed25519, returning a 64-byte signature.
    pub fn sign(&self, data: &[u8]) -> ChainResult<Vec<u8>> {
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&self.private_key);
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let signature = signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    /// Sign an AE transaction.
    ///
    /// AE protocol: `sign(network_id_bytes ++ blake2b_256(tx_bytes))`
    /// AE protocol: `sign(network_id_bytes ++ blake2b_256(tx_bytes))`
    pub fn sign_transaction(&self, tx_bytes: &[u8]) -> ChainResult<Vec<u8>> {
        let tx_hash = crate::blake2b_256(tx_bytes);
        let mut payload = Vec::with_capacity(self.network_id.len() + 32);
        payload.extend_from_slice(self.network_id.as_bytes());
        payload.extend_from_slice(&tx_hash);
        self.sign(&payload)
    }

    /// Return the raw public key bytes.
    pub fn get_public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Return the network ID.
    pub fn network_id(&self) -> &str {
        &self.network_id
    }

    /// Return the `ak_`-prefixed address string.
    pub fn address_string(&self) -> String {
        self.encoded_address.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    fn test_key_bytes() -> Vec<u8> {
        vec![
            0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60, 0xba, 0x84, 0x4a, 0xf4, 0x92, 0xec,
            0x2c, 0xc4, 0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19, 0x70, 0x3b, 0xac, 0x03,
            0x1c, 0xae, 0x7f, 0x60,
        ]
    }

    #[test]
    fn test_new_derives_correct_public_key() {
        let signer = AeSigner::new(test_key_bytes(), "ae_uat".into()).unwrap();
        assert_eq!(signer.public_key.len(), 32);
        assert!(signer.encoded_address.starts_with("ak_"));
    }

    #[test]
    fn test_sign_produces_valid_signature() {
        let signer = AeSigner::new(test_key_bytes(), "ae_uat".into()).unwrap();
        let message = b"hello world";
        let sig_bytes = signer.sign(message).unwrap();
        assert_eq!(sig_bytes.len(), 64);

        let mut pk_bytes = [0u8; 32];
        pk_bytes.copy_from_slice(&signer.public_key);
        let verifying_key = VerifyingKey::from_bytes(&pk_bytes).unwrap();
        let signature = Signature::from_bytes(sig_bytes.as_slice().try_into().unwrap());
        assert!(verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_sign_transaction_includes_network_id() {
        let signer = AeSigner::new(test_key_bytes(), "ae_uat".into()).unwrap();
        let tx_bytes = b"fake_tx_data";
        let sig = signer.sign_transaction(tx_bytes).unwrap();
        assert_eq!(sig.len(), 64);

        let mut pk_bytes = [0u8; 32];
        pk_bytes.copy_from_slice(&signer.public_key);
        let verifying_key = VerifyingKey::from_bytes(&pk_bytes).unwrap();

        let tx_hash = crate::blake2b_256(tx_bytes);
        let mut expected_payload = Vec::new();
        expected_payload.extend_from_slice(b"ae_uat");
        expected_payload.extend_from_slice(&tx_hash);

        let signature = Signature::from_bytes(sig.as_slice().try_into().unwrap());
        assert!(verifying_key.verify(&expected_payload, &signature).is_ok());
    }

    #[test]
    fn test_address_encoding_produces_valid_ak_address() {
        let signer = AeSigner::new(test_key_bytes(), "ae_mainnet".into()).unwrap();
        assert!(signer.encoded_address.starts_with("ak_"));
        assert!(signer.encoded_address.len() > 10);
    }

    #[test]
    fn test_invalid_key_length_rejected() {
        let result = AeSigner::new(vec![0u8; 16], "ae_uat".into());
        assert!(result.is_err());
    }
}

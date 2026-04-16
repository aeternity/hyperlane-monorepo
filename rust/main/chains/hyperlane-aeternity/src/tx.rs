use rlp::RlpStream;
use sha2::{Digest, Sha256};

use hyperlane_core::ChainResult;

use crate::signer::AeSigner;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CONTRACT_CALL_TX_TAG: u64 = 42;
const CONTRACT_CALL_TX_VERSION: u64 = 1;
const SIGNED_TX_TAG: u64 = 11;
const SIGNED_TX_VERSION: u64 = 1;
const FATE_ABI_VERSION: u64 = 3;

const GAS_PER_BYTE: u64 = 20;
const BASE_GAS_PRICE: u64 = 1_000_000_000;
const SIGNATURE_SIZE: u64 = 64;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// RLP-encoded + signed transaction ready for submission.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SignedTransaction {
    /// Transaction hash (`th_...` prefixed)
    pub hash: String,
    /// Base64-encoded signed tx (`tx_...` prefixed)
    pub encoded: String,
    /// Gas limit used
    pub gas: u64,
    /// Fee included
    pub fee: u64,
}

/// Outcome of a submitted + confirmed transaction.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TxOutcome {
    /// Transaction hash
    pub txid: String,
    /// Gas actually consumed
    pub gas_used: u64,
    /// Whether the call succeeded
    pub success: bool,
}

// ---------------------------------------------------------------------------
// AeTxBuilder
// ---------------------------------------------------------------------------

/// Builds, signs, and submits Aeternity contract call transactions.
#[derive(Debug)]
pub struct AeTxBuilder {
    signer: AeSigner,
    #[allow(dead_code)]
    network_id: String,
}

impl AeTxBuilder {
    pub fn new(signer: AeSigner, network_id: String) -> Self {
        Self { signer, network_id }
    }

    /// Build and sign a contract call transaction.
    ///
    /// Returns a `SignedTransaction` with the RLP-encoded + base64 tx and hash.
    pub fn build_contract_call(
        &self,
        caller_id: &[u8],
        nonce: u64,
        contract_id: &[u8],
        fee: u64,
        amount: u64,
        gas: u64,
        gas_price: u64,
        call_data: &[u8],
    ) -> ChainResult<SignedTransaction> {
        let inner_tx = rlp_encode_contract_call(
            caller_id,
            nonce,
            contract_id,
            FATE_ABI_VERSION,
            fee,
            0, // ttl = 0 (no TTL)
            amount,
            gas,
            gas_price,
            call_data,
        );

        let signature = self.signer.sign_transaction(&inner_tx)?;
        let signed_tx = rlp_encode_signed_tx(vec![signature], &inner_tx);

        let tx_hash = Sha256::digest(Sha256::digest(&signed_tx));
        let hash_str = crate::utils::encode_ae_hash(&tx_hash, "th")?;

        use base64::{engine::general_purpose::STANDARD, Engine};
        let encoded = format!("tx_{}", STANDARD.encode(&signed_tx));

        Ok(SignedTransaction {
            hash: hash_str,
            encoded,
            gas,
            fee,
        })
    }

    /// Calculate the minimum fee for a given transaction byte size.
    pub fn calculate_fee(tx_byte_size: u64) -> u64 {
        (tx_byte_size + SIGNATURE_SIZE) * GAS_PER_BYTE * BASE_GAS_PRICE
    }

    /// Return a reference to the inner signer.
    #[allow(dead_code)]
    pub fn signer(&self) -> &AeSigner {
        &self.signer
    }
}

// ---------------------------------------------------------------------------
// RLP encoding
// ---------------------------------------------------------------------------

/// RLP-encode a ContractCallTx with tag=42, version=1.
fn rlp_encode_contract_call(
    caller_id: &[u8],
    nonce: u64,
    contract_id: &[u8],
    abi_version: u64,
    fee: u64,
    ttl: u64,
    amount: u64,
    gas: u64,
    gas_price: u64,
    call_data: &[u8],
) -> Vec<u8> {
    let mut stream = RlpStream::new_list(11);
    stream.append(&CONTRACT_CALL_TX_TAG);
    stream.append(&CONTRACT_CALL_TX_VERSION);
    stream.append(&caller_id);
    stream.append(&nonce);
    stream.append(&contract_id);
    stream.append(&abi_version);
    stream.append(&fee);
    stream.append(&ttl);
    stream.append(&amount);
    stream.append(&gas);
    stream.append(&gas_price);
    stream.append(&call_data);
    stream.out().to_vec()
}

/// RLP-encode a SignedTx with tag=11, version=1.
fn rlp_encode_signed_tx(signatures: Vec<Vec<u8>>, tx_bytes: &[u8]) -> Vec<u8> {
    let mut stream = RlpStream::new_list(4);
    stream.append(&SIGNED_TX_TAG);
    stream.append(&SIGNED_TX_VERSION);
    stream.append_list::<Vec<u8>, Vec<u8>>(&signatures);
    stream.append(&tx_bytes);
    stream.out().to_vec()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_signer() -> AeSigner {
        let key = vec![
            0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60, 0xba, 0x84, 0x4a, 0xf4, 0x92, 0xec,
            0x2c, 0xc4, 0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19, 0x70, 0x3b, 0xac, 0x03,
            0x1c, 0xae, 0x7f, 0x60,
        ];
        AeSigner::new(key, "ae_uat".into()).unwrap()
    }

    #[test]
    fn test_fee_calculation() {
        let fee = AeTxBuilder::calculate_fee(100);
        // (100 + 64) * 20 * 1_000_000_000 = 164 * 20_000_000_000 = 3_280_000_000_000
        assert_eq!(fee, 3_280_000_000_000);
    }

    #[test]
    fn test_rlp_contract_call_encoding() {
        let encoded = rlp_encode_contract_call(
            &[1u8; 32],
            1,
            &[2u8; 32],
            FATE_ABI_VERSION,
            100_000,
            0,
            0,
            100_000,
            BASE_GAS_PRICE,
            b"calldata",
        );
        assert!(!encoded.is_empty());
        // RLP-encoded list should start with a list prefix
        assert!(encoded[0] >= 0xc0 || encoded[0] >= 0xf7);
    }

    #[test]
    fn test_rlp_signed_tx_encoding() {
        let sig = vec![0xABu8; 64];
        let tx = vec![0xCD; 100];
        let encoded = rlp_encode_signed_tx(vec![sig], &tx);
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_build_contract_call() {
        let signer = test_signer();
        let builder = AeTxBuilder::new(signer, "ae_uat".into());

        let result = builder.build_contract_call(
            &[1u8; 32],
            1,
            &[2u8; 32],
            100_000,
            0,
            100_000,
            BASE_GAS_PRICE,
            b"calldata",
        );

        assert!(result.is_ok());
        let signed_tx = result.unwrap();
        assert!(signed_tx.hash.starts_with("th_"));
        assert!(signed_tx.encoded.starts_with("tx_"));
    }
}

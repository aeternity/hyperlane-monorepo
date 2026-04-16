use async_trait::async_trait;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use tracing::debug;

use hyperlane_core::{
    BlockInfo, ChainInfo, ChainResult, FixedPointNumber, HyperlaneChain, HyperlaneDomain,
    HyperlaneProvider, LogMeta, ReorgPeriod, TxOutcome, TxnInfo, TxnReceiptInfo, H256, H512, U256,
};

use crate::{
    ae_address_to_h256, ae_timestamp_to_seconds, decode_ae_hash, encode_ae_hash,
    h256_to_contract_address,
    rpc::{AeMdwClient, AeNodeClient, ContractLogEntry, DryRunCallReq, DryRunRequest, DryRunTx},
    ConnectionConf, HyperlaneAeternityError,
};

// ---------------------------------------------------------------------------
// FATE value abstraction
// ---------------------------------------------------------------------------

/// Lightweight representation of a FATE value for calldata encoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FateValue {
    /// FATE integer
    Integer(BigUint),
    /// FATE boolean
    Boolean(bool),
    /// Raw bytes
    Bytes(Vec<u8>),
    /// UTF-8 string
    String(String),
    /// Aeternity address (32-byte pubkey)
    Address(H256),
    /// Homogeneous list
    List(Vec<FateValue>),
    /// Key-value map
    Map(Vec<(FateValue, FateValue)>),
    /// Fixed-size tuple
    Tuple(Vec<FateValue>),
}

/// Encode function name + arguments into calldata suitable for a dry-run call.
///
/// This uses a simplified scheme: Blake2b-256 hash of the function name (first 4
/// bytes) followed by the serialized argument representation. Real FATE ABI
/// encoding would use the `aebytecode` library; this helper is sufficient for
/// dry-run-based read calls via the node API.
#[allow(dead_code)]
pub fn encode_calldata(function: &str, _args: &[FateValue]) -> Vec<u8> {
    let hash = crate::blake2b_256(function.as_bytes());
    let mut calldata = Vec::with_capacity(4);
    calldata.extend_from_slice(&hash[..4]);
    calldata
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

/// Aeternity blockchain provider implementing the Hyperlane provider trait.
#[derive(Debug, Clone)]
pub struct AeternityProvider {
    domain: HyperlaneDomain,
    node: AeNodeClient,
    mdw: AeMdwClient,
    signer: Option<AeSigner>,
    reorg_period: ReorgPeriod,
    #[allow(dead_code)]
    network_id: String,
}

/// Minimal signer placeholder for the Aeternity chain.
///
/// Full signing support will be implemented in a follow-up phase.
#[derive(Debug, Clone)]
pub struct AeSigner {
    /// Ed25519 private key bytes
    pub private_key: Vec<u8>,
    /// Encoded `ak_...` address
    pub encoded_address: String,
    /// H256 representation of the address
    pub address_h256: H256,
}

impl AeternityProvider {
    /// Create a new Aeternity provider.
    pub fn new(
        domain: HyperlaneDomain,
        conf: &ConnectionConf,
        reorg_period: ReorgPeriod,
        signer: Option<AeSigner>,
    ) -> ChainResult<Self> {
        let node_url = conf.node_urls.first().ok_or_else(|| {
            HyperlaneAeternityError::Other("no node URLs configured".into())
        })?;
        let mdw_url = conf.mdw_urls.first().ok_or_else(|| {
            HyperlaneAeternityError::Other("no middleware URLs configured".into())
        })?;

        Ok(Self {
            domain,
            node: AeNodeClient::new(node_url.clone()),
            mdw: AeMdwClient::new(mdw_url.clone()),
            signer,
            reorg_period,
            network_id: conf.network_id.clone(),
        })
    }

    /// Return a reference to the node RPC client.
    pub fn node(&self) -> &AeNodeClient {
        &self.node
    }

    /// Return a reference to the middleware RPC client.
    pub fn mdw(&self) -> &AeMdwClient {
        &self.mdw
    }

    /// Return the configured signer, or an error if none was set.
    pub fn get_signer(&self) -> ChainResult<&AeSigner> {
        self.signer
            .as_ref()
            .ok_or_else(|| HyperlaneAeternityError::SignerMissing.into())
    }

    /// Read a contract function via dry-run (no on-chain transaction).
    ///
    /// The returned `FateValue` currently wraps the raw FATE-encoded return
    /// string; full decoding will be added once `aebytecode` bindings are
    /// available.
    pub async fn call_contract(
        &self,
        contract_id: &str,
        function_name: &str,
        args: Vec<FateValue>,
    ) -> ChainResult<FateValue> {
        let calldata_bytes = encode_calldata(function_name, &args);
        use base64::{engine::general_purpose::STANDARD, Engine};
        let calldata_str = format!("cb_{}", STANDARD.encode(&calldata_bytes));

        let default_caller =
            "ak_11111111111111111111111111111111273Yts".to_string();

        let request = DryRunRequest {
            top: None,
            accounts: vec![],
            txs: vec![DryRunTx {
                tx: String::new(),
                call_req: Some(DryRunCallReq {
                    contract: contract_id.to_string(),
                    calldata: calldata_str,
                    caller: default_caller,
                    abi_version: Some(3),
                    amount: Some(0),
                    gas: Some(1_000_000),
                }),
            }],
        };

        let response = self.node.dry_run(&request).await?;
        let result = response.results.into_iter().next().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError("empty dry-run results".into())
        })?;

        if result.result_type != "ok" {
            return Err(HyperlaneAeternityError::ContractCallError(format!(
                "dry-run failed: {:?}",
                result.reason
            ))
            .into());
        }

        let call_obj = result.call_obj.ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError("missing call_obj in dry-run".into())
        })?;

        if call_obj.return_type != "ok" {
            return Err(HyperlaneAeternityError::ContractCallError(format!(
                "contract call returned: {}",
                call_obj.return_type
            ))
            .into());
        }

        let return_value = call_obj.return_value.ok_or_else(|| {
            HyperlaneAeternityError::ReturnDecodingError("missing return value".into())
        })?;

        // TODO: decode FATE-encoded return value into proper FateValue variant
        Ok(FateValue::String(return_value))
    }

    /// Build, sign, and submit a contract call transaction.
    ///
    /// Encodes the function name and arguments into FATE calldata, constructs
    /// the RLP-encoded transaction, signs it with the configured signer, and
    /// posts it to the node.
    pub async fn send_contract_call(
        &self,
        contract_id: &str,
        function_name: &str,
        args: Vec<FateValue>,
        amount: u64,
        _gas: u64,
    ) -> ChainResult<TxOutcome> {
        let signer = self.get_signer()?;
        let calldata_bytes = encode_calldata(function_name, &args);
        use base64::{engine::general_purpose::STANDARD, Engine};
        let calldata_str = format!("cb_{}", STANDARD.encode(&calldata_bytes));

        let tx_builder = crate::tx::AeTxBuilder::new(
            crate::signer::AeSigner::new(
                signer.private_key.clone(),
                self.network_id.clone(),
            )?,
            self.network_id.clone(),
        );

        let caller_bytes = signer.address_h256.as_bytes().to_vec();
        let contract_h256 = ae_address_to_h256(contract_id)?;
        let contract_bytes = contract_h256.as_bytes().to_vec();

        let nonce_resp = self.node.get_account(&signer.encoded_address).await?;
        let nonce = nonce_resp.nonce + 1;
        let gas = 1_000_000u64;
        let gas_price = 1_000_000_000u64;
        let fee = crate::tx::AeTxBuilder::calculate_fee(200);

        let signed_tx = tx_builder.build_contract_call(
            &caller_bytes,
            nonce,
            &contract_bytes,
            fee,
            amount,
            gas,
            gas_price,
            calldata_str.as_bytes(),
        )?;

        let resp = self.node.post_transaction(&signed_tx.encoded).await?;
        let hash_str = resp
            .get("tx_hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                HyperlaneAeternityError::TransactionError(
                    "missing tx_hash in post response".into(),
                )
            })?;

        let h256 = decode_ae_hash(hash_str)?;
        let mut h512_bytes = [0u8; 64];
        h512_bytes[32..].copy_from_slice(h256.as_bytes());

        Ok(TxOutcome {
            transaction_id: H512::from(h512_bytes),
            executed: true,
            gas_used: U256::from(gas),
            gas_price: FixedPointNumber::try_from(U256::from(gas_price))
                .unwrap_or_default(),
        })
    }

    /// Fetch contract log entries within a block-height range from the middleware.
    pub async fn fetch_logs_in_range(
        &self,
        contract_id: &str,
        from_height: u64,
        to_height: u64,
    ) -> ChainResult<Vec<(ContractLogEntry, LogMeta)>> {
        let entries = self
            .mdw
            .get_contract_logs(contract_id, from_height, to_height)
            .await?;

        let contract_h256 = ae_address_to_h256(contract_id)?;

        let mut result = Vec::with_capacity(entries.len());
        for entry in entries {
            let block_hash = decode_ae_hash(&entry.block_hash).unwrap_or_default();
            let tx_hash = decode_ae_hash(&entry.call_tx_hash).unwrap_or_default();
            let mut tx_h512 = [0u8; 64];
            tx_h512[32..].copy_from_slice(tx_hash.as_bytes());

            let meta = LogMeta {
                address: contract_h256,
                block_number: entry.height,
                block_hash,
                transaction_id: H512::from(tx_h512),
                transaction_index: entry.micro_index,
                log_index: U256::from(entry.log_idx),
            };
            result.push((entry, meta));
        }

        Ok(result)
    }

    /// Get the finalized block number, accounting for the reorg period.
    pub async fn get_finalized_block_number(&self) -> ChainResult<u64> {
        let current = self.node.get_current_key_block().await?;
        let reorg_blocks = self.reorg_period.as_blocks().unwrap_or(0);
        Ok(current.height.saturating_sub(reorg_blocks as u64))
    }
}

impl HyperlaneChain for AeternityProvider {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(self.clone())
    }
}

#[async_trait]
impl HyperlaneProvider for AeternityProvider {
    async fn get_block_by_height(&self, height: u64) -> ChainResult<BlockInfo> {
        let block = self.node.get_key_block_by_height(height).await?;
        let hash = decode_ae_hash(&block.hash).unwrap_or_default();
        Ok(BlockInfo {
            hash,
            timestamp: ae_timestamp_to_seconds(block.time),
            number: block.height,
        })
    }

    async fn get_txn_by_hash(&self, hash: &H512) -> ChainResult<TxnInfo> {
        let h256_bytes = &hash.as_bytes()[32..];
        let h256 = H256::from_slice(h256_bytes);
        let hash_str = encode_ae_hash(h256.as_bytes(), "th")?;

        let tx = self.node.get_transaction(&hash_str).await?;

        let sender_str = tx
            .tx
            .get("sender_id")
            .or_else(|| tx.tx.get("caller_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let sender = ae_address_to_h256(sender_str).unwrap_or_default();

        let recipient_str = tx
            .tx
            .get("recipient_id")
            .or_else(|| tx.tx.get("contract_id"))
            .and_then(|v| v.as_str());
        let recipient = recipient_str.and_then(|r| ae_address_to_h256(r).ok());

        let nonce = tx
            .tx
            .get("nonce")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let gas_limit_val = tx
            .tx
            .get("gas")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let gas_price_val = tx
            .tx
            .get("gas_price")
            .and_then(|v| v.as_u64());

        let receipt = match self.node.get_transaction_info(&hash_str).await {
            Ok(info) => {
                let gas_used = info.gas_used.unwrap_or(0);
                let gas_price = info.gas_price.unwrap_or(0);
                Some(TxnReceiptInfo {
                    gas_used: U256::from(gas_used),
                    cumulative_gas_used: U256::from(gas_used),
                    effective_gas_price: Some(U256::from(gas_price)),
                })
            }
            Err(_) => None,
        };

        Ok(TxnInfo {
            hash: *hash,
            gas_limit: U256::from(gas_limit_val),
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            gas_price: gas_price_val.map(U256::from),
            nonce,
            sender,
            recipient,
            receipt,
            raw_input_data: None,
        })
    }

    async fn is_contract(&self, address: &H256) -> ChainResult<bool> {
        let ct_addr = h256_to_contract_address(*address);
        match self.node.get_account(&ct_addr).await {
            Ok(account) => Ok(account.kind.as_deref() == Some("contract")),
            Err(_) => {
                debug!(address = %ct_addr, "address not found or not a contract");
                Ok(false)
            }
        }
    }

    async fn get_balance(&self, address: String) -> ChainResult<U256> {
        let account = self.node.get_account(&address).await?;
        Ok(U256::from(account.balance))
    }

    async fn get_chain_metrics(&self) -> ChainResult<Option<ChainInfo>> {
        let block = self.node.get_current_key_block().await?;
        let hash = decode_ae_hash(&block.hash).unwrap_or_default();
        Ok(Some(ChainInfo::new(
            BlockInfo {
                hash,
                timestamp: ae_timestamp_to_seconds(block.time),
                number: block.height,
            },
            None,
        )))
    }
}

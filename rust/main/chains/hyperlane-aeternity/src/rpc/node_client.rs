use std::time::Instant;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use hyperlane_core::ChainResult;
use hyperlane_metric::prometheus_metric::{
    ChainInfo, ClientConnectionType, PrometheusClientMetrics, PrometheusConfig,
};

use crate::HyperlaneAeternityError;

/// HTTP client for the Aeternity node REST API (`/v3/...`).
#[derive(Debug, Clone)]
pub struct AeNodeClient {
    client: Client,
    base_url: Url,
    metrics: PrometheusClientMetrics,
    config: PrometheusConfig,
}

// ---------------------------------------------------------------------------
// Response / request types
// ---------------------------------------------------------------------------

/// A key-block as returned by the node API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBlockResponse {
    /// Block hash
    pub hash: String,
    /// Block height
    pub height: u64,
    /// Previous block hash
    pub prev_hash: String,
    /// Previous key-block hash
    pub prev_key_hash: String,
    /// Miner address
    pub miner: Option<String>,
    /// Beneficiary address
    pub beneficiary: Option<String>,
    /// Unix timestamp in milliseconds
    pub time: u64,
    /// Protocol version
    pub version: u64,
}

/// A micro-block header within a generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroBlockHeader {
    /// Micro-block hash
    pub hash: String,
    /// Parent key-block hash
    pub prev_hash: String,
    /// Unix timestamp in milliseconds
    pub time: u64,
    /// Height (same as key-block height)
    pub height: u64,
    /// Proof-of-fraud hash (if applicable)
    pub pof_hash: Option<String>,
    /// Transaction count
    pub txs_hash: Option<String>,
}

/// A generation (key-block + its micro-blocks).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    /// The key block for this generation
    pub key_block: KeyBlockResponse,
    /// Micro block headers belonging to this generation
    pub micro_blocks: Vec<String>,
}

/// A transaction as returned by the node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    /// Transaction hash
    pub hash: String,
    /// Signature list
    pub signatures: Vec<String>,
    /// Transaction body
    pub tx: serde_json::Value,
    /// Block hash (present once mined)
    pub block_hash: Option<String>,
    /// Block height (present once mined)
    pub block_height: Option<u64>,
}

/// Detailed transaction info (result of a mined tx).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfoResponse {
    /// Call info (for contract calls)
    pub call_info: Option<CallInfoResponse>,
    /// Gas used
    pub gas_used: Option<u64>,
    /// Gas price
    pub gas_price: Option<u64>,
    /// Return type (ok | revert | error)
    pub return_type: Option<String>,
}

/// Contract call result details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallInfoResponse {
    /// Return type (ok | revert | error)
    pub return_type: String,
    /// Return value (FATE encoded, base64)
    pub return_value: Option<String>,
    /// Gas used by the call
    pub gas_used: u64,
    /// Gas price
    pub gas_price: u64,
    /// Emitted log entries
    pub log: Vec<CallLogEntry>,
    /// Contract address
    pub contract_id: Option<String>,
    /// Caller address
    pub caller_id: Option<String>,
}

/// A single log entry emitted during a contract call.
///
/// The AE node returns event topics as raw FATE integers (which can be
/// 256-bit values), so `topics` and `data` use `serde_json::Value` to
/// avoid deserialization failures on very large numbers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallLogEntry {
    /// Contract address that emitted the event
    pub address: String,
    /// Event topics (hashed event name + indexed args, returned as large ints)
    pub topics: Vec<serde_json::Value>,
    /// Non-indexed event data (FATE-encoded)
    pub data: serde_json::Value,
}

/// An account as returned by the node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResponse {
    /// Account address
    pub id: String,
    /// Account balance in aettos (can exceed u64 on devnets).
    pub balance: serde_json::Number,
    /// Current nonce
    pub nonce: u64,
    /// Account kind (basic | generalized)
    pub kind: Option<String>,
}

impl AccountResponse {
    /// Best-effort conversion of balance to u128 (saturates on overflow).
    pub fn balance_u128(&self) -> u128 {
        if let Some(v) = self.balance.as_u64() {
            return v as u128;
        }
        if let Some(f) = self.balance.as_f64() {
            return f as u128;
        }
        0
    }
}

/// Request body for the `/v3/dry-run` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunRequest {
    /// Top block hash to simulate against (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<String>,
    /// Accounts to inject into state
    pub accounts: Vec<DryRunAccount>,
    /// Transactions to simulate
    pub txs: Vec<DryRunTx>,
}

/// An account override for dry-run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunAccount {
    /// Account public key
    pub pub_key: String,
    /// Override balance
    pub amount: u64,
}

/// A transaction to simulate in a dry-run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunTx {
    /// Base64-encoded transaction (omit when using `call_req`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx: Option<String>,
    /// Call-specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_req: Option<DryRunCallReq>,
}

/// Fields for a contract call within a dry-run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunCallReq {
    /// Contract address
    pub contract: String,
    /// Encoded calldata
    pub calldata: String,
    /// Caller address
    pub caller: String,
    /// ABI version (default 3 for FATE)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi_version: Option<u32>,
    /// Transfer amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,
    /// Gas limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<u64>,
    /// Caller nonce — must be >= next valid nonce for the caller account
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,
}

/// Response from the `/v3/dry-run` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunResponse {
    /// Results for each submitted transaction
    pub results: Vec<DryRunResult>,
}

/// A single dry-run result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunResult {
    /// The kind of dry-run entry (e.g. `"contract_call"`)
    #[serde(rename = "type")]
    pub result_type: String,
    /// Whether the call succeeded (`"ok"` or `"error"`)
    pub result: String,
    /// Call object (present on success)
    pub call_obj: Option<CallInfoResponse>,
    /// Reason string (present on failure)
    pub reason: Option<String>,
}

/// Response from the `/v3/accounts/{address}/next-nonce` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextNonceResponse {
    /// The next valid nonce for this account
    pub next_nonce: u64,
}

/// Node status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatusResponse {
    /// Network identifier
    pub network_id: String,
    /// Node version
    pub node_version: String,
    /// Node revision (git SHA)
    pub node_revision: Option<String>,
    /// Genesis key-block hash
    pub genesis_key_block_hash: String,
    /// Top key-block hash
    pub top_key_block_hash: String,
    /// Top block height
    pub top_block_height: u64,
}

// ---------------------------------------------------------------------------
// Client implementation
// ---------------------------------------------------------------------------

impl AeNodeClient {
    /// Create a new node client pointing at `base_url`.
    pub fn new(base_url: Url, metrics: PrometheusClientMetrics, chain: Option<ChainInfo>) -> Self {
        let config = PrometheusConfig::from_url(&base_url, ClientConnectionType::Rpc, chain);
        Self {
            client: Client::new(),
            base_url,
            metrics,
            config,
        }
    }

    /// Build a full URL for the given API path.
    fn url(&self, path: &str) -> String {
        format!(
            "{}/v3{}",
            self.base_url.as_str().trim_end_matches('/'),
            path
        )
    }

    fn track(&self, method: &str, start: Instant, success: bool) {
        self.metrics
            .increment_metrics(&self.config, method, start, success);
    }

    /// Fetch a key-block by its height.
    pub async fn get_key_block_by_height(&self, height: u64) -> ChainResult<KeyBlockResponse> {
        let start = Instant::now();
        let url = self.url(&format!("/key-blocks/height/{height}"));
        let res: ChainResult<_> = async {
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(HyperlaneAeternityError::from)?;
            let status = resp.status();
            let body = resp.text().await.map_err(HyperlaneAeternityError::from)?;
            if !status.is_success() {
                return Err(HyperlaneAeternityError::NodeApiError(format!(
                    "GET {url} => {status}: {body}"
                ))
                .into());
            }
            serde_json::from_str(&body).map_err(|e| HyperlaneAeternityError::from(e).into())
        }
        .await;
        self.track("get_key_block_by_height", start, res.is_ok());
        res
    }

    /// Fetch the current (latest) key-block.
    pub async fn get_current_key_block(&self) -> ChainResult<KeyBlockResponse> {
        let start = Instant::now();
        let url = self.url("/key-blocks/current");
        let res: ChainResult<_> = async {
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(HyperlaneAeternityError::from)?;
            let status = resp.status();
            let body = resp.text().await.map_err(HyperlaneAeternityError::from)?;
            if !status.is_success() {
                return Err(HyperlaneAeternityError::NodeApiError(format!(
                    "GET {url} => {status}: {body}"
                ))
                .into());
            }
            serde_json::from_str(&body).map_err(|e| HyperlaneAeternityError::from(e).into())
        }
        .await;
        self.track("get_current_key_block", start, res.is_ok());
        res
    }

    /// Fetch a full generation (key-block + micro-block hashes) by height.
    pub async fn get_generation_by_height(&self, height: u64) -> ChainResult<GenerationResponse> {
        let start = Instant::now();
        let url = self.url(&format!("/generations/height/{height}"));
        let res: ChainResult<_> = async {
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(HyperlaneAeternityError::from)?;
            let status = resp.status();
            let body = resp.text().await.map_err(HyperlaneAeternityError::from)?;
            if !status.is_success() {
                return Err(HyperlaneAeternityError::NodeApiError(format!(
                    "GET {url} => {status}: {body}"
                ))
                .into());
            }
            serde_json::from_str(&body).map_err(|e| HyperlaneAeternityError::from(e).into())
        }
        .await;
        self.track("get_generation_by_height", start, res.is_ok());
        res
    }

    /// Fetch a transaction by its hash.
    pub async fn get_transaction(&self, hash: &str) -> ChainResult<TransactionResponse> {
        let start = Instant::now();
        let url = self.url(&format!("/transactions/{hash}"));
        let res: ChainResult<_> = async {
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(HyperlaneAeternityError::from)?;
            let status = resp.status();
            let body = resp.text().await.map_err(HyperlaneAeternityError::from)?;
            if !status.is_success() {
                return Err(HyperlaneAeternityError::NodeApiError(format!(
                    "GET {url} => {status}: {body}"
                ))
                .into());
            }
            serde_json::from_str(&body).map_err(|e| HyperlaneAeternityError::from(e).into())
        }
        .await;
        self.track("get_transaction", start, res.is_ok());
        res
    }

    /// Fetch detailed info about a mined transaction.
    pub async fn get_transaction_info(&self, hash: &str) -> ChainResult<TransactionInfoResponse> {
        let start = Instant::now();
        let url = self.url(&format!("/transactions/{hash}/info"));
        let res: ChainResult<_> = async {
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(HyperlaneAeternityError::from)?;
            let status = resp.status();
            let body = resp.text().await.map_err(HyperlaneAeternityError::from)?;
            if !status.is_success() {
                return Err(HyperlaneAeternityError::NodeApiError(format!(
                    "GET {url} => {status}: {body}"
                ))
                .into());
            }
            serde_json::from_str(&body).map_err(|e| HyperlaneAeternityError::from(e).into())
        }
        .await;
        self.track("get_transaction_info", start, res.is_ok());
        res
    }

    /// Post a signed transaction to the node mempool.
    pub async fn post_transaction(&self, signed_tx: &str) -> ChainResult<serde_json::Value> {
        let start = Instant::now();
        let url = self.url("/transactions");
        let body = serde_json::json!({ "tx": signed_tx });
        let res: ChainResult<_> = async {
            let resp = self
                .client
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(HyperlaneAeternityError::from)?;
            let status = resp.status();
            let text = resp.text().await.map_err(HyperlaneAeternityError::from)?;
            if !status.is_success() {
                return Err(HyperlaneAeternityError::NodeApiError(format!(
                    "POST {url} => {status}: {text}"
                ))
                .into());
            }
            serde_json::from_str(&text).map_err(|e| HyperlaneAeternityError::from(e).into())
        }
        .await;
        self.track("post_transaction", start, res.is_ok());
        res
    }

    /// Fetch account information (balance, nonce, etc.).
    pub async fn get_account(&self, address: &str) -> ChainResult<AccountResponse> {
        let start = Instant::now();
        let url = self.url(&format!("/accounts/{address}"));
        let res: ChainResult<_> = async {
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(HyperlaneAeternityError::from)?;
            let status = resp.status();
            let body = resp.text().await.map_err(HyperlaneAeternityError::from)?;
            if !status.is_success() {
                return Err(HyperlaneAeternityError::NodeApiError(format!(
                    "GET {url} => {status}: {body}"
                ))
                .into());
            }
            serde_json::from_str(&body).map_err(|e| HyperlaneAeternityError::from(e).into())
        }
        .await;
        self.track("get_account", start, res.is_ok());
        res
    }

    /// Check whether a `ct_`-prefixed address actually refers to a deployed contract.
    /// Uses `/v3/contracts/{address}` which returns 200 for contracts and 404 otherwise.
    pub async fn contract_exists(&self, ct_address: &str) -> ChainResult<bool> {
        let start = Instant::now();
        let url = self.url(&format!("/contracts/{ct_address}"));
        let res: ChainResult<bool> = async {
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(HyperlaneAeternityError::from)?;
            Ok(resp.status().is_success())
        }
        .await;
        self.track("contract_exists", start, res.is_ok());
        res
    }

    /// Fetch the next valid nonce for the given account.
    pub async fn get_next_nonce(&self, address: &str) -> ChainResult<u64> {
        let start = Instant::now();
        let url = self.url(&format!("/accounts/{address}/next-nonce"));
        let res: ChainResult<_> = async {
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(HyperlaneAeternityError::from)?;
            let status = resp.status();
            let body = resp.text().await.map_err(HyperlaneAeternityError::from)?;
            if !status.is_success() {
                return Ok(1);
            }
            let parsed: NextNonceResponse =
                serde_json::from_str(&body).map_err(|e| HyperlaneAeternityError::from(e))?;
            Ok(parsed.next_nonce)
        }
        .await;
        self.track("get_next_nonce", start, res.is_ok());
        res
    }

    /// Execute a dry-run simulation against the node.
    pub async fn dry_run(&self, request: &DryRunRequest) -> ChainResult<DryRunResponse> {
        let start = Instant::now();
        let url = self.url("/dry-run");
        let res: ChainResult<_> = async {
            let resp = self
                .client
                .post(&url)
                .json(request)
                .send()
                .await
                .map_err(HyperlaneAeternityError::from)?;
            let status = resp.status();
            let body = resp.text().await.map_err(HyperlaneAeternityError::from)?;
            if !status.is_success() {
                return Err(HyperlaneAeternityError::NodeApiError(format!(
                    "POST {url} => {status}: {body}"
                ))
                .into());
            }
            serde_json::from_str(&body).map_err(|e| HyperlaneAeternityError::from(e).into())
        }
        .await;
        self.track("dry_run", start, res.is_ok());
        res
    }

    /// Fetch node status (network id, top block height, etc.).
    pub async fn get_status(&self) -> ChainResult<NodeStatusResponse> {
        let start = Instant::now();
        let url = self.url("/status");
        let res: ChainResult<_> = async {
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(HyperlaneAeternityError::from)?;
            let status = resp.status();
            let body = resp.text().await.map_err(HyperlaneAeternityError::from)?;
            if !status.is_success() {
                return Err(HyperlaneAeternityError::NodeApiError(format!(
                    "GET {url} => {status}: {body}"
                ))
                .into());
            }
            serde_json::from_str(&body).map_err(|e| HyperlaneAeternityError::from(e).into())
        }
        .await;
        self.track("get_status", start, res.is_ok());
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_block_response_deserialization() {
        let json = r#"{
            "hash": "kh_test",
            "height": 100,
            "prev_hash": "kh_prev",
            "prev_key_hash": "kh_prev_key",
            "miner": null,
            "beneficiary": null,
            "time": 1700000000000,
            "version": 6
        }"#;
        let block: KeyBlockResponse = serde_json::from_str(json).unwrap();
        assert_eq!(block.height, 100);
        assert_eq!(block.hash, "kh_test");
    }

    #[test]
    fn test_account_response_deserialization() {
        let json = r#"{
            "id": "ak_test",
            "balance": 1000000,
            "nonce": 5,
            "kind": "basic"
        }"#;
        let account: AccountResponse = serde_json::from_str(json).unwrap();
        assert_eq!(account.balance_u128(), 1_000_000);
        assert_eq!(account.nonce, 5);
    }

    #[test]
    fn test_dry_run_request_serialization() {
        let req = DryRunRequest {
            top: None,
            accounts: vec![],
            txs: vec![],
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"accounts\":[]"));
        assert!(!json.contains("\"top\""), "top: None must be omitted");
    }

    #[test]
    fn test_node_status_deserialization() {
        let json = r#"{
            "network_id": "ae_mainnet",
            "node_version": "7.0.0",
            "node_revision": "abc123",
            "genesis_key_block_hash": "kh_genesis",
            "top_key_block_hash": "kh_top",
            "top_block_height": 500000
        }"#;
        let status: NodeStatusResponse = serde_json::from_str(json).unwrap();
        assert_eq!(status.network_id, "ae_mainnet");
        assert_eq!(status.top_block_height, 500_000);
    }
}

use std::time::Instant;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use hyperlane_core::ChainResult;
use hyperlane_metric::prometheus_metric::{
    ChainInfo, ClientConnectionType, PrometheusClientMetrics, PrometheusConfig,
};

use crate::HyperlaneAeternityError;

/// A single contract log entry from the middleware.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractLogEntry {
    /// Contract that emitted the event
    pub contract_id: String,
    /// Transaction hash of the call that produced this log
    pub call_tx_hash: String,
    /// Block hash
    pub block_hash: String,
    /// Key-block height
    pub height: u64,
    /// Micro-block index within the generation
    #[serde(default)]
    pub micro_index: u64,
    /// Log index within the micro-block
    #[serde(default)]
    pub log_idx: u64,
    /// Blake2b hash of the event name
    pub event_hash: Option<String>,
    /// Event arguments (topic values)
    #[serde(default)]
    pub args: Vec<String>,
    /// Non-indexed event data
    #[serde(default)]
    pub data: String,
}

/// Paginated response wrapper used by the middleware.
#[derive(Debug, Clone, Deserialize)]
struct PaginatedResponse<T> {
    data: Vec<T>,
    #[serde(default)]
    next: Option<String>,
}

/// HTTP client for the Aeternity middleware REST API (`/mdw/v3/...`).
#[derive(Debug, Clone)]
pub struct AeMdwClient {
    client: Client,
    base_url: Url,
    metrics: PrometheusClientMetrics,
    config: PrometheusConfig,
}

impl AeMdwClient {
    /// Create a new middleware client pointing at `base_url`.
    pub fn new(
        base_url: Url,
        metrics: PrometheusClientMetrics,
        chain: Option<ChainInfo>,
    ) -> Self {
        let config = PrometheusConfig::from_url(&base_url, ClientConnectionType::Rpc, chain);
        Self {
            client: Client::new(),
            base_url,
            metrics,
            config,
        }
    }

    /// Build a full URL for the given middleware path.
    fn url(&self, path: &str) -> String {
        format!(
            "{}/mdw/v3{}",
            self.base_url.as_str().trim_end_matches('/'),
            path
        )
    }

    fn track(&self, method: &str, start: Instant, success: bool) {
        self.metrics
            .increment_metrics(&self.config, method, start, success);
    }

    /// Fetch contract log entries for a specific contract within a height range.
    ///
    /// Automatically follows pagination cursors to collect all matching entries.
    pub async fn get_contract_logs(
        &self,
        contract_id: &str,
        from_height: u64,
        to_height: u64,
    ) -> ChainResult<Vec<ContractLogEntry>> {
        let start = Instant::now();
        let res: ChainResult<_> = async {
            let mut all_entries = Vec::new();
            let mut next_url: Option<String> = None;

            loop {
                let url = match &next_url {
                    Some(cursor) => format!(
                        "{}/mdw/v3{}",
                        self.base_url.as_str().trim_end_matches('/'),
                        cursor
                    ),
                    None => {
                        let from = from_height.max(1);
                        let path = format!(
                            "/contracts/logs?contract_id={contract_id}&scope=gen:{from}-{to_height}&limit=100"
                        );
                        self.url(&path)
                    }
                };

                let resp = self
                    .client
                    .get(&url)
                    .send()
                    .await
                    .map_err(HyperlaneAeternityError::from)?;

                let status = resp.status();
                let body = resp.text().await.map_err(HyperlaneAeternityError::from)?;
                if !status.is_success() {
                    return Err(HyperlaneAeternityError::MiddlewareApiError(format!(
                        "GET {url} => {status}: {body}"
                    ))
                    .into());
                }

                let page: PaginatedResponse<ContractLogEntry> =
                    serde_json::from_str(&body).map_err(HyperlaneAeternityError::from)?;

                all_entries.extend(page.data);

                match page.next {
                    Some(cursor) if !cursor.is_empty() => next_url = Some(cursor),
                    _ => break,
                }
            }

            Ok(all_entries)
        }.await;
        self.track("get_contract_logs", start, res.is_ok());
        res
    }

    /// Fetch the current chain height from the middleware status endpoint.
    pub async fn get_current_height(&self) -> ChainResult<u64> {
        let start = Instant::now();
        let res: ChainResult<_> = async {
            let url = format!(
                "{}/mdw/v3/status",
                self.base_url.as_str().trim_end_matches('/')
            );
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(HyperlaneAeternityError::from)?;
            let status = resp.status();
            let body = resp.text().await.map_err(HyperlaneAeternityError::from)?;
            if !status.is_success() {
                return Err(HyperlaneAeternityError::MiddlewareApiError(format!(
                    "GET {url} => {status}: {body}"
                ))
                .into());
            }

            let value: serde_json::Value =
                serde_json::from_str(&body).map_err(HyperlaneAeternityError::from)?;

            let height = value
                .get("mdw_height")
                .or_else(|| value.get("height"))
                .and_then(|v| v.as_u64())
                .ok_or_else(|| {
                    HyperlaneAeternityError::MiddlewareApiError(
                        "missing height in status response".into(),
                    )
                })?;

            Ok(height)
        }.await;
        self.track("get_current_height", start, res.is_ok());
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_log_entry_deserialization() {
        let json = r#"{
            "contract_id": "ct_test",
            "call_tx_hash": "th_test",
            "block_hash": "mh_test",
            "height": 42,
            "micro_index": 0,
            "log_idx": 1,
            "event_hash": "abc123",
            "args": ["arg1", "arg2"],
            "data": "some_data"
        }"#;
        let entry: ContractLogEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.height, 42);
        assert_eq!(entry.args.len(), 2);
    }

    #[test]
    fn test_contract_log_entry_defaults() {
        let json = r#"{
            "contract_id": "ct_test",
            "call_tx_hash": "th_test",
            "block_hash": "mh_test",
            "height": 1
        }"#;
        let entry: ContractLogEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.micro_index, 0);
        assert!(entry.args.is_empty());
        assert!(entry.data.is_empty());
    }
}

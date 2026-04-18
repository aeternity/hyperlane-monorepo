use std::collections::HashMap;
use std::time::Instant;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use hyperlane_core::ChainResult;
use hyperlane_metric::prometheus_metric::{
    ChainInfo, ClientConnectionType, PrometheusClientMetrics, PrometheusConfig,
};

use crate::HyperlaneAeternityError;

/// HTTP client for the Sophia compiler (`aesophia_http`).
///
/// Provides FATE calldata encoding and return value decoding by delegating
/// to the compiler's `/encode-calldata` and `/decode-call-result` endpoints.
/// This is the same approach the JS SDK and `aeproject` use.
#[derive(Debug, Clone)]
pub struct AeCompilerClient {
    client: Client,
    base_url: Url,
    metrics: PrometheusClientMetrics,
    config: PrometheusConfig,
}

#[derive(Debug, Serialize)]
struct EncodeCalldataRequest {
    source: String,
    function: String,
    arguments: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<CompilerOptions>,
}

#[derive(Debug, Serialize)]
struct DecodeCallResultRequest {
    source: String,
    function: String,
    #[serde(rename = "call-result")]
    call_result: String,
    #[serde(rename = "call-value")]
    call_value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<CompilerOptions>,
}

#[derive(Debug, Serialize)]
struct CompilerOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    file_system: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct EncodeCalldataResponse {
    calldata: String,
}

impl AeCompilerClient {
    /// Create a new compiler client pointing at `base_url`.
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

    fn track(&self, method: &str, start: Instant, success: bool) {
        self.metrics
            .increment_metrics(&self.config, method, start, success);
    }

    /// Encode calldata for a contract function call.
    ///
    /// Sends the contract source, function name, and Sophia-formatted arguments
    /// to the compiler's `/encode-calldata` endpoint. Returns the `cb_...`
    /// encoded calldata string suitable for dry-run or transaction submission.
    pub async fn encode_calldata(
        &self,
        source: &str,
        function: &str,
        arguments: &[String],
        file_system: Option<&HashMap<String, String>>,
    ) -> ChainResult<String> {
        let start = Instant::now();
        let res: ChainResult<_> = async {
            let url = format!(
                "{}/encode-calldata",
                self.base_url.as_str().trim_end_matches('/')
            );

            let options = file_system.map(|fs| CompilerOptions {
                file_system: Some(fs.clone()),
            });

            let body = EncodeCalldataRequest {
                source: source.to_string(),
                function: function.to_string(),
                arguments: arguments.to_vec(),
                options,
            };

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
                return Err(HyperlaneAeternityError::CalldataEncodingError(format!(
                    "POST {url} => {status}: {text}"
                ))
                .into());
            }

            let parsed: EncodeCalldataResponse =
                serde_json::from_str(&text).map_err(HyperlaneAeternityError::from)?;
            Ok(parsed.calldata)
        }.await;
        self.track("encode_calldata", start, res.is_ok());
        res
    }

    /// Decode the return value from a contract call.
    ///
    /// Sends the contract source, function name, call result status, and the
    /// `cb_...` encoded return value to the compiler's `/decode-call-result`
    /// endpoint. Returns the decoded value as a `serde_json::Value`.
    pub async fn decode_call_result(
        &self,
        source: &str,
        function: &str,
        call_result: &str,
        call_value: &str,
        file_system: Option<&HashMap<String, String>>,
    ) -> ChainResult<serde_json::Value> {
        let start = Instant::now();
        let res: ChainResult<_> = async {
            let url = format!(
                "{}/decode-call-result",
                self.base_url.as_str().trim_end_matches('/')
            );

            let options = file_system.map(|fs| CompilerOptions {
                file_system: Some(fs.clone()),
            });

            let body = DecodeCallResultRequest {
                source: source.to_string(),
                function: function.to_string(),
                call_result: call_result.to_string(),
                call_value: call_value.to_string(),
                options,
            };

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
                return Err(HyperlaneAeternityError::ReturnDecodingError(format!(
                    "POST {url} => {status}: {text}"
                ))
                .into());
            }

            serde_json::from_str(&text).map_err(|e| HyperlaneAeternityError::from(e).into())
        }.await;
        self.track("decode_call_result", start, res.is_ok());
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_calldata_request_serialization() {
        let body = EncodeCalldataRequest {
            source: "contract C = entrypoint f() = 1".to_string(),
            function: "f".to_string(),
            arguments: vec![],
            options: None,
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"source\""));
        assert!(json.contains("\"function\""));
        assert!(!json.contains("\"options\""));
    }

    #[test]
    fn test_decode_call_result_request_serialization() {
        let body = DecodeCallResultRequest {
            source: "contract C = entrypoint f() = 1".to_string(),
            function: "f".to_string(),
            call_result: "ok".to_string(),
            call_value: "cb_test".to_string(),
            options: None,
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"call-result\""));
        assert!(json.contains("\"call-value\""));
    }

    #[test]
    fn test_encode_calldata_request_with_filesystem() {
        let mut fs = HashMap::new();
        fs.insert("IFoo.aes".to_string(), "contract interface IFoo = entrypoint bar : () => int".to_string());

        let body = EncodeCalldataRequest {
            source: "include \"IFoo.aes\"\ncontract C = entrypoint f() = 1".to_string(),
            function: "f".to_string(),
            arguments: vec![],
            options: Some(CompilerOptions {
                file_system: Some(fs),
            }),
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"file_system\""));
        assert!(json.contains("IFoo.aes"));
    }
}

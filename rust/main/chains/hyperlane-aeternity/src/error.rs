use hyperlane_core::ChainCommunicationError;

/// Errors from the crates specific to hyperlane-aeternity
#[derive(Debug, thiserror::Error)]
pub enum HyperlaneAeternityError {
    /// Reqwest HTTP client error
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    /// JSON serialization / deserialization error
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    /// Standard I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Error returned by the Aeternity node HTTP API
    #[error("Node API error: {0}")]
    NodeApiError(String),
    /// Error returned by the Aeternity middleware HTTP API
    #[error("Middleware API error: {0}")]
    MiddlewareApiError(String),
    /// Failed to encode FATE calldata
    #[error("Calldata encoding error: {0}")]
    CalldataEncodingError(String),
    /// Failed to decode a FATE return value
    #[error("Return decoding error: {0}")]
    ReturnDecodingError(String),
    /// Invalid or un-parseable address
    #[error("Address error: {0}")]
    AddressError(String),
    /// RLP encoding / decoding error
    #[error("RLP error: {0}")]
    RlpError(String),
    /// Cryptographic signing failed
    #[error("Signing error: {0}")]
    SigningError(String),
    /// Transaction submission or confirmation error
    #[error("Transaction error: {0}")]
    TransactionError(String),
    /// Timed out waiting for transaction confirmation
    #[error("Transaction timeout: {0}")]
    TransactionTimeout(String),
    /// A contract call returned an error
    #[error("Contract call error: {0}")]
    ContractCallError(String),
    /// Failed to parse a contract event
    #[error("Event parse error: {0}")]
    EventParseError(String),
    /// Signer is required but was not configured
    #[error("Signer missing")]
    SignerMissing,
    /// Catch-all for other errors
    #[error("{0}")]
    Other(String),
}

impl From<HyperlaneAeternityError> for ChainCommunicationError {
    fn from(value: HyperlaneAeternityError) -> Self {
        ChainCommunicationError::from_other(value)
    }
}

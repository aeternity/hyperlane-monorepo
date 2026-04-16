use url::Url;

/// Aeternity connection configuration
#[derive(Clone, Debug)]
pub struct ConnectionConf {
    /// Aeternity node RPC endpoints (`/v3/...`)
    pub node_urls: Vec<Url>,
    /// Aeternity middleware endpoints (`/mdw/v3/...`)
    pub mdw_urls: Vec<Url>,
    /// Network identifier (e.g. `ae_mainnet`, `ae_uat`)
    pub network_id: String,
}

impl ConnectionConf {
    /// Create a new connection configuration.
    pub fn new(node_urls: Vec<Url>, mdw_urls: Vec<Url>, network_id: String) -> Self {
        Self {
            node_urls,
            mdw_urls,
            network_id,
        }
    }
}

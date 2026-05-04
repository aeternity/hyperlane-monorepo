use hyperlane_core::ChainResult;

use crate::{contracts, AeternityProvider, HyperlaneAeternityError};

/// Read-only wrapper for InterchainQueryRouter queries.
/// Used for monitoring and debugging ICQ operations.
#[derive(Debug)]
pub struct AeInterchainQueryRouter {
    provider: AeternityProvider,
    contract_address: String,
}

impl AeInterchainQueryRouter {
    /// Creates a new ICQ router query wrapper.
    pub fn new(provider: AeternityProvider, contract_address: String) -> Self {
        Self {
            provider,
            contract_address,
        }
    }

    /// Get the deployment block height.
    pub async fn deployed_block(&self) -> ChainResult<u64> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "deployed_block",
                &[],
                &contracts::INTERCHAIN_QUERY_ROUTER_SOURCE,
            )
            .await?;

        result.as_u64().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(
                "expected integer from deployed_block()".into(),
            )
            .into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icq_router_source_exists() {
        assert!(!contracts::INTERCHAIN_QUERY_ROUTER_SOURCE.is_empty());
        assert!(contracts::INTERCHAIN_QUERY_ROUTER_SOURCE.contains("InterchainQueryRouterStub"));
    }
}

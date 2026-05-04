use hyperlane_core::{ChainResult, H256};

use crate::{contracts, AeternityProvider, HyperlaneAeternityError};

/// Read-only wrapper for InterchainAccountRouter queries.
/// Used for monitoring and debugging ICA deployments.
#[derive(Debug)]
pub struct AeInterchainAccountRouter {
    provider: AeternityProvider,
    contract_address: String,
}

impl AeInterchainAccountRouter {
    /// Creates a new ICA router query wrapper.
    pub fn new(provider: AeternityProvider, contract_address: String) -> Self {
        Self {
            provider,
            contract_address,
        }
    }

    /// Query the local ICA address for a given (origin, owner) pair.
    pub async fn get_local_ica(&self, origin: u32, owner: H256) -> ChainResult<Option<String>> {
        let owner_hex = format!("#{}", hex::encode(owner.as_bytes()));
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "get_local_ica",
                &[origin.to_string(), owner_hex],
                &contracts::INTERCHAIN_ACCOUNT_ROUTER_SOURCE,
            )
            .await;

        match result {
            Ok(val) => {
                if val.is_null() || val.as_str() == Some("None") {
                    Ok(None)
                } else {
                    Ok(val.as_str().map(String::from))
                }
            }
            Err(_) => Ok(None),
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
                &contracts::INTERCHAIN_ACCOUNT_ROUTER_SOURCE,
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
    fn test_ica_router_source_exists() {
        assert!(!contracts::INTERCHAIN_ACCOUNT_ROUTER_SOURCE.is_empty());
        assert!(contracts::INTERCHAIN_ACCOUNT_ROUTER_SOURCE.contains("InterchainAccountRouterStub"));
    }
}

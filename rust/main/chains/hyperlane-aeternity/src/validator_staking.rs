use hyperlane_core::{ChainResult, ContractLocator, H256, U256};

use crate::{
    contracts, h256_to_account_address, h256_to_contract_address, AeternityProvider,
    HyperlaneAeternityError,
};

/// Wrapper for the ValidatorStaking contract.
/// Provides validator stake status queries for the relayer and validator agents.
#[derive(Debug)]
pub struct AeValidatorStaking {
    provider: AeternityProvider,
    contract_address: String,
}

impl AeValidatorStaking {
    /// Creates a new ValidatorStaking wrapper.
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address);
        Ok(Self {
            provider,
            contract_address,
        })
    }

    /// Check if a validator is currently active (staked above min_stake, not unstaking).
    pub async fn is_active_validator(&self, validator: H256) -> ChainResult<bool> {
        let addr = h256_to_account_address(validator);
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "is_active_validator",
                &[addr],
                &contracts::VALIDATOR_STAKING_SOURCE,
            )
            .await?;

        result.as_bool().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(
                "expected bool from is_active_validator()".into(),
            )
            .into()
        })
    }

    /// Get total staked amount across all validators.
    pub async fn get_total_staked(&self) -> ChainResult<U256> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "get_total_staked",
                &[],
                &contracts::VALIDATOR_STAKING_SOURCE,
            )
            .await?;

        Ok(U256::from(result.as_u64().unwrap_or(0)))
    }

    /// Get the deployment block height for indexing.
    pub async fn deployed_block(&self) -> ChainResult<u64> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "deployed_block",
                &[],
                &contracts::VALIDATOR_STAKING_SOURCE,
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
    fn test_validator_staking_source_exists() {
        assert!(!contracts::VALIDATOR_STAKING_SOURCE.is_empty());
        assert!(contracts::VALIDATOR_STAKING_SOURCE.contains("ValidatorStakingStub"));
    }
}

use hyperlane_core::{ChainResult, ContractLocator, H256};

use crate::{contracts, h256_to_contract_address, AeternityProvider, HyperlaneAeternityError};

/// Wrapper for CheckpointFraudProofs contract.
/// Provides on-chain fraud detection queries for premature and invalid checkpoints.
#[derive(Debug)]
pub struct AeCheckpointFraudProofs {
    provider: AeternityProvider,
    contract_address: String,
}

impl AeCheckpointFraudProofs {
    /// Creates a new CheckpointFraudProofs wrapper.
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address);
        Ok(Self {
            provider,
            contract_address,
        })
    }

    /// Check if a checkpoint is premature (index exceeds actual tree count).
    pub async fn is_premature(
        &self,
        checkpoint_root: H256,
        checkpoint_index: u32,
        merkle_tree_addr: &str,
    ) -> ChainResult<bool> {
        let root_hex = format!("#{}", hex::encode(checkpoint_root.as_bytes()));
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "is_premature",
                &[
                    root_hex,
                    checkpoint_index.to_string(),
                    merkle_tree_addr.to_string(),
                ],
                &contracts::CHECKPOINT_FRAUD_PROOFS_SOURCE,
            )
            .await?;

        result.as_bool().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError("expected bool from is_premature()".into())
                .into()
        })
    }

    /// Get the deployment block height for indexing.
    pub async fn deployed_block(&self) -> ChainResult<u64> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "deployed_block",
                &[],
                &contracts::CHECKPOINT_FRAUD_PROOFS_SOURCE,
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
    fn test_checkpoint_fraud_proofs_source_exists() {
        assert!(!contracts::CHECKPOINT_FRAUD_PROOFS_SOURCE.is_empty());
        assert!(contracts::CHECKPOINT_FRAUD_PROOFS_SOURCE.contains("CheckpointFraudProofsStub"));
    }
}

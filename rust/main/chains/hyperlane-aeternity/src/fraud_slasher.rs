use hyperlane_core::{ChainResult, ContractLocator, TxOutcome, H256};

use crate::{contracts, h256_to_account_address, h256_to_contract_address, AeternityProvider};

/// Wrapper for the FraudSlasher contract.
/// Enables permissionless slashing of validators with proven checkpoint fraud.
#[derive(Debug)]
pub struct AeFraudSlasher {
    provider: AeternityProvider,
    contract_address: String,
}

impl AeFraudSlasher {
    /// Creates a new FraudSlasher wrapper.
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address);
        Ok(Self {
            provider,
            contract_address,
        })
    }

    /// Slash a validator for premature checkpoint fraud.
    pub async fn slash_premature(&self, validator: H256, proof_id: H256) -> ChainResult<TxOutcome> {
        let validator_addr = h256_to_account_address(validator);
        let proof_hex = format!("#{}", hex::encode(proof_id.as_bytes()));

        self.provider
            .send_contract_call(
                &self.contract_address,
                "slash_premature",
                &[validator_addr, proof_hex],
                &contracts::FRAUD_SLASHER_SOURCE,
                0,
                0,
            )
            .await
    }

    /// Slash a validator for fraudulent message ID.
    pub async fn slash_fraudulent_message_id(
        &self,
        validator: H256,
        proof_id: H256,
    ) -> ChainResult<TxOutcome> {
        let validator_addr = h256_to_account_address(validator);
        let proof_hex = format!("#{}", hex::encode(proof_id.as_bytes()));

        self.provider
            .send_contract_call(
                &self.contract_address,
                "slash_fraudulent_message_id",
                &[validator_addr, proof_hex],
                &contracts::FRAUD_SLASHER_SOURCE,
                0,
                0,
            )
            .await
    }

    /// Slash a validator for fraudulent root.
    pub async fn slash_fraudulent_root(
        &self,
        validator: H256,
        proof_id: H256,
    ) -> ChainResult<TxOutcome> {
        let validator_addr = h256_to_account_address(validator);
        let proof_hex = format!("#{}", hex::encode(proof_id.as_bytes()));

        self.provider
            .send_contract_call(
                &self.contract_address,
                "slash_fraudulent_root",
                &[validator_addr, proof_hex],
                &contracts::FRAUD_SLASHER_SOURCE,
                0,
                0,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fraud_slasher_source_exists() {
        assert!(!contracts::FRAUD_SLASHER_SOURCE.is_empty());
        assert!(contracts::FRAUD_SLASHER_SOURCE.contains("FraudSlasherStub"));
    }
}

use async_trait::async_trait;

use hyperlane_core::{
    Announcement, ChainResult, ContractLocator, HyperlaneChain, HyperlaneContract,
    HyperlaneDomain, HyperlaneProvider, SignedType, TxOutcome, ValidatorAnnounce, H256, U256,
};

use crate::{
    h256_to_contract_address, AeternityProvider, FateValue, HyperlaneAeternityError,
};

/// Aeternity Validator Announce
///
/// Validators use this contract to publish their checkpoint storage locations.
/// The contract stores `(validator_address, storage_location)` pairs and verifies
/// EIP-191 signatures on-chain.
#[derive(Debug)]
pub struct AeValidatorAnnounce {
    provider: AeternityProvider,
    contract_address: String,
    address_h256: H256,
    domain: HyperlaneDomain,
}

impl AeValidatorAnnounce {
    /// Creates a new Aeternity ValidatorAnnounce instance
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address)?;
        Ok(Self {
            domain: provider.domain().clone(),
            provider,
            contract_address,
            address_h256: locator.address,
        })
    }

    /// Extract the last 20 bytes from H256 as an Ethereum-style validator address.
    ///
    /// Hyperlane identifies validators by their secp256k1 Ethereum address (20 bytes)
    /// stored in H256 (32 bytes, left-padded with 12 zero bytes).
    fn h256_to_eth_address_bytes(h: &H256) -> Vec<u8> {
        h.as_bytes()[12..].to_vec()
    }
}

impl HyperlaneContract for AeValidatorAnnounce {
    fn address(&self) -> H256 {
        self.address_h256
    }
}

impl HyperlaneChain for AeValidatorAnnounce {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(self.provider.clone())
    }
}

#[async_trait]
impl ValidatorAnnounce for AeValidatorAnnounce {
    /// Returns the announced storage locations for the provided validators.
    ///
    /// For each validator (identified by secp256k1 Ethereum-style address as H256),
    /// returns the list of storage location strings they've announced.
    ///
    /// Calls Sophia entrypoint:
    ///   `get_announced_storage_locations(validators: list(bytes(20))) : list(list(string))`
    async fn get_announced_storage_locations(
        &self,
        validators: &[H256],
    ) -> ChainResult<Vec<Vec<String>>> {
        let validator_bytes: Vec<FateValue> = validators
            .iter()
            .map(|v| FateValue::Bytes(Self::h256_to_eth_address_bytes(v)))
            .collect();

        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "get_announced_storage_locations",
                vec![FateValue::List(validator_bytes)],
            )
            .await?;

        match result {
            FateValue::List(outer) => {
                let mut all_locations = Vec::with_capacity(outer.len());
                for inner_value in outer {
                    match inner_value {
                        FateValue::List(inner) => {
                            let mut locations = Vec::with_capacity(inner.len());
                            for item in inner {
                                match item {
                                    FateValue::String(s) => locations.push(s),
                                    other => {
                                        return Err(
                                            HyperlaneAeternityError::ContractCallError(
                                                format!(
                                                "expected String for storage location, got {:?}",
                                                other
                                            ),
                                            )
                                            .into(),
                                        )
                                    }
                                }
                            }
                            all_locations.push(locations);
                        }
                        other => {
                            return Err(HyperlaneAeternityError::ContractCallError(format!(
                                "expected List for validator locations, got {:?}",
                                other
                            ))
                            .into())
                        }
                    }
                }
                Ok(all_locations)
            }
            other => Err(HyperlaneAeternityError::ContractCallError(format!(
                "expected List from get_announced_storage_locations(), got {:?}",
                other
            ))
            .into()),
        }
    }

    /// Announce a validator's storage location.
    ///
    /// The announcement must be signed with the validator's secp256k1 key.
    /// The signature is verified on-chain by the ValidatorAnnounce contract.
    ///
    /// Calls Sophia entrypoint:
    ///   `announce(validator: bytes(20), storage_location: string, signature: bytes(65))`
    async fn announce(&self, announcement: SignedType<Announcement>) -> ChainResult<TxOutcome> {
        let validator_bytes =
            Self::h256_to_eth_address_bytes(&announcement.value.validator.into());
        let storage_location = announcement.value.storage_location.clone();
        let signature = announcement.signature.to_vec();

        self.provider
            .send_contract_call(
                &self.contract_address,
                "announce",
                vec![
                    FateValue::Bytes(validator_bytes),
                    FateValue::String(storage_location),
                    FateValue::Bytes(signature),
                ],
                0,
                0,
            )
            .await
    }

    /// Returns the number of additional tokens needed to pay for the announce
    /// transaction.
    ///
    /// Announce on Aeternity only costs gas — no additional tokens are required.
    async fn announce_tokens_needed(
        &self,
        _announcement: SignedType<Announcement>,
        _chain_signer: H256,
    ) -> Option<U256> {
        Some(U256::zero())
    }
}

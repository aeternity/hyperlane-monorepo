use async_trait::async_trait;

use hyperlane_core::{
    Announcement, ChainResult, ContractLocator, HyperlaneChain, HyperlaneContract,
    HyperlaneDomain, HyperlaneProvider, SignedType, TxOutcome, ValidatorAnnounce, H256, U256,
};

use crate::{
    contracts, h256_to_contract_address, AeternityProvider, HyperlaneAeternityError,
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
        let contract_address = h256_to_contract_address(locator.address);
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
    fn h256_to_eth_address_hex(h: &H256) -> String {
        format!("#{}", hex::encode(&h.as_bytes()[12..]))
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
    /// Calls Sophia entrypoint:
    ///   `get_announced_storage_locations(validators: list(bytes(20))) : list(list(string))`
    async fn get_announced_storage_locations(
        &self,
        validators: &[H256],
    ) -> ChainResult<Vec<Vec<String>>> {
        let validator_args: Vec<String> = validators
            .iter()
            .map(Self::h256_to_eth_address_hex)
            .collect();
        let list_arg = format!("[{}]", validator_args.join(", "));

        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "get_announced_storage_locations",
                &[list_arg],
                contracts::VALIDATOR_ANNOUNCE_SOURCE,
            )
            .await?;

        let outer = result.as_array().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected list from get_announced_storage_locations(), got {result}"
            ))
        })?;

        let mut all_locations = Vec::with_capacity(outer.len());
        for inner_value in outer {
            let inner = inner_value.as_array().ok_or_else(|| {
                HyperlaneAeternityError::ContractCallError(format!(
                    "expected list for validator locations, got {inner_value}"
                ))
            })?;
            let mut locations = Vec::with_capacity(inner.len());
            for item in inner {
                let s = item.as_str().ok_or_else(|| {
                    HyperlaneAeternityError::ContractCallError(format!(
                        "expected string for storage location, got {item}"
                    ))
                })?;
                locations.push(s.to_string());
            }
            all_locations.push(locations);
        }
        Ok(all_locations)
    }

    /// Announce a validator's storage location.
    async fn announce(&self, announcement: SignedType<Announcement>) -> ChainResult<TxOutcome> {
        let validator_hex = Self::h256_to_eth_address_hex(&announcement.value.validator.into());
        let storage_location = format!("\"{}\"", announcement.value.storage_location);
        let signature_hex = format!("#{}", hex::encode(announcement.signature.to_vec()));

        self.provider
            .send_contract_call(
                &self.contract_address,
                "announce",
                &[validator_hex, storage_location, signature_hex],
                contracts::VALIDATOR_ANNOUNCE_SOURCE,
                0,
                0,
            )
            .await
    }

    /// Returns the number of additional tokens needed to pay for the announce
    /// transaction.
    async fn announce_tokens_needed(
        &self,
        _announcement: SignedType<Announcement>,
        _chain_signer: H256,
    ) -> Option<U256> {
        Some(U256::zero())
    }
}

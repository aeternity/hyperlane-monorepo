use async_trait::async_trait;

use hyperlane_core::{
    ChainResult, ContractLocator, Encode, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneMessage, HyperlaneProvider, InterchainSecurityModule, Metadata, ModuleType,
    RoutingIsm, H256, U256,
};

use super::routing_ism::parse_route_result;
use crate::{contracts, h256_to_contract_address, AeternityProvider};

/// AmountRoutingIsm — routes to different ISMs based on token transfer amount.
///
/// Small transfers (below threshold) use `lower_ism`;
/// large transfers (at or above threshold) use `upper_ism`.
/// From the relayer's perspective, behavior is identical to DomainRoutingIsm.
#[derive(Debug)]
pub struct AeAmountRoutingIsm {
    provider: AeternityProvider,
    contract_address: String,
    address_h256: H256,
    domain: HyperlaneDomain,
}

impl AeAmountRoutingIsm {
    /// Creates a new AmountRoutingIsm backed by an Aeternity contract.
    pub fn new(provider: AeternityProvider, locator: &ContractLocator) -> ChainResult<Self> {
        let contract_address = h256_to_contract_address(locator.address);
        Ok(Self {
            domain: provider.domain().clone(),
            provider,
            contract_address,
            address_h256: locator.address,
        })
    }
}

impl HyperlaneContract for AeAmountRoutingIsm {
    fn address(&self) -> H256 {
        self.address_h256
    }
}

impl HyperlaneChain for AeAmountRoutingIsm {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(self.provider.clone())
    }
}

#[async_trait]
impl InterchainSecurityModule for AeAmountRoutingIsm {
    async fn module_type(&self) -> ChainResult<ModuleType> {
        Ok(ModuleType::Routing)
    }

    async fn dry_run_verify(
        &self,
        _message: &HyperlaneMessage,
        _metadata: &Metadata,
    ) -> ChainResult<Option<U256>> {
        Ok(None)
    }
}

#[async_trait]
impl RoutingIsm for AeAmountRoutingIsm {
    /// Route to the appropriate ISM based on message transfer amount.
    async fn route(&self, message: &HyperlaneMessage) -> ChainResult<H256> {
        let message_hex = format!("Bytes.to_any_size(#{})", hex::encode(message.to_vec()));
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "route",
                &[message_hex],
                &contracts::AMOUNT_ROUTING_ISM_SOURCE,
            )
            .await?;

        parse_route_result(&result)
    }
}

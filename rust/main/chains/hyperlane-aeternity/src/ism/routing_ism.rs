use async_trait::async_trait;

use hyperlane_core::{
    ChainResult, ContractLocator, Encode, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneMessage, HyperlaneProvider, InterchainSecurityModule, Metadata, ModuleType,
    RoutingIsm, H256, U256,
};

use crate::{
    ae_address_to_h256, contracts, h256_to_contract_address, AeternityProvider,
    HyperlaneAeternityError,
};

use super::interchain_security_module::json_to_module_type;

/// Aeternity Routing ISM
///
/// Wraps `DomainRoutingIsm.aes` — routes to a domain-specific ISM based on the
/// message's origin domain.
#[derive(Debug)]
pub struct AeRoutingIsm {
    provider: AeternityProvider,
    contract_address: String,
    address_h256: H256,
    domain: HyperlaneDomain,
}

impl AeRoutingIsm {
    /// Creates a new Aeternity RoutingIsm instance
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

impl HyperlaneContract for AeRoutingIsm {
    fn address(&self) -> H256 {
        self.address_h256
    }
}

impl HyperlaneChain for AeRoutingIsm {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(self.provider.clone())
    }
}

#[async_trait]
impl InterchainSecurityModule for AeRoutingIsm {
    async fn module_type(&self) -> ChainResult<ModuleType> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "module_type",
                &[],
                &contracts::ROUTING_ISM_SOURCE,
            )
            .await?;

        json_to_module_type(&result)
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
impl RoutingIsm for AeRoutingIsm {
    /// Returns the ISM address that should verify this message.
    ///
    /// Calls Sophia entrypoint: `route(message: bytes()) : IInterchainSecurityModule`
    async fn route(&self, message: &HyperlaneMessage) -> ChainResult<H256> {
        let message_hex = format!("#{}", hex::encode(message.to_vec()));

        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "route",
                &[message_hex],
                &contracts::ROUTING_ISM_SOURCE,
            )
            .await?;

        let addr_str = result.as_str().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError(format!(
                "expected address string from route(), got {result}"
            ))
        })?;

        ae_address_to_h256(addr_str)
    }
}

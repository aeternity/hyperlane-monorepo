use async_trait::async_trait;

use hyperlane_core::{
    ChainResult, ContractLocator, Encode, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneMessage, HyperlaneProvider, InterchainSecurityModule, Metadata, ModuleType,
    RoutingIsm, H256, U256,
};

use crate::{
    h256_to_contract_address, AeternityProvider, FateValue,
    HyperlaneAeternityError,
};

use super::interchain_security_module::fate_to_module_type;

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
    /// Returns the module type of this ISM.
    async fn module_type(&self) -> ChainResult<ModuleType> {
        let result = self
            .provider
            .call_contract(&self.contract_address, "module_type", vec![])
            .await?;

        fate_to_module_type(result)
    }

    /// Dry runs the `verify()` ISM call.
    ///
    /// Returns `None` — Aeternity handles verification on-chain.
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
    /// The contract examines the message's origin domain and returns the
    /// registered ISM for that domain, or falls back to default.
    ///
    /// Calls Sophia entrypoint: `route(message: bytes) : IInterchainSecurityModule`
    async fn route(&self, message: &HyperlaneMessage) -> ChainResult<H256> {
        let message_bytes = message.to_vec();

        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "route",
                vec![FateValue::Bytes(message_bytes)],
            )
            .await?;

        match result {
            FateValue::Address(addr) => Ok(addr),
            other => Err(HyperlaneAeternityError::ContractCallError(format!(
                "expected Address from route(), got {:?}",
                other
            ))
            .into()),
        }
    }
}

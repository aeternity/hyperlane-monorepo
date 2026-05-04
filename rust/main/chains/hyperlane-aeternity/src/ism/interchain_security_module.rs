use async_trait::async_trait;
use tracing::warn;

use hyperlane_core::{
    ChainResult, ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneMessage, HyperlaneProvider, InterchainSecurityModule, Metadata, ModuleType, H256,
    U256,
};

use crate::{contracts, h256_to_contract_address, AeternityProvider, HyperlaneAeternityError};

/// Aeternity Interchain Security Module
///
/// Base ISM implementation used when the agent needs to interact with an ISM
/// at a known address but doesn't know the specific type yet.
#[derive(Debug)]
pub struct AeIsm {
    provider: AeternityProvider,
    contract_address: String,
    address_h256: H256,
    domain: HyperlaneDomain,
}

impl AeIsm {
    /// Creates a new Aeternity ISM instance
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

/// Map an integer module type from the compiler-decoded JSON to a `ModuleType` enum.
pub(crate) fn json_to_module_type(value: &serde_json::Value) -> ChainResult<ModuleType> {
    let type_id = value.as_u64().ok_or_else(|| {
        HyperlaneAeternityError::ContractCallError(format!(
            "expected integer from module_type(), got {value}"
        ))
    })? as u32;

    match type_id {
        0 => Ok(ModuleType::Unused),
        1 => Ok(ModuleType::Routing),
        2 => Ok(ModuleType::Aggregation),
        3 => Ok(ModuleType::LegacyMultisig),
        4 => Ok(ModuleType::MerkleRootMultisig),
        5 => Ok(ModuleType::MessageIdMultisig),
        6 => Ok(ModuleType::Null),
        7 => Ok(ModuleType::CcipRead),
        // Weighted multisig variants (Hyperlane v3) — map to base multisig types
        // since the relayer uses identical metadata format for verification.
        9 => Ok(ModuleType::MerkleRootMultisig),
        10 => Ok(ModuleType::MessageIdMultisig),
        n => {
            warn!(
                module_type = n,
                "unknown ISM module type, treating as Unused"
            );
            Ok(ModuleType::Unused)
        }
    }
}

impl HyperlaneContract for AeIsm {
    fn address(&self) -> H256 {
        self.address_h256
    }
}

impl HyperlaneChain for AeIsm {
    fn domain(&self) -> &HyperlaneDomain {
        &self.domain
    }

    fn provider(&self) -> Box<dyn HyperlaneProvider> {
        Box::new(self.provider.clone())
    }
}

#[async_trait]
impl InterchainSecurityModule for AeIsm {
    async fn module_type(&self) -> ChainResult<ModuleType> {
        let result = self
            .provider
            .call_contract(
                &self.contract_address,
                "module_type",
                &[],
                &contracts::BASE_ISM_SOURCE,
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

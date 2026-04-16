use async_trait::async_trait;
use num_traits::ToPrimitive;

use hyperlane_core::{
    ChainResult, ContractLocator, HyperlaneChain, HyperlaneContract, HyperlaneDomain,
    HyperlaneMessage, HyperlaneProvider, InterchainSecurityModule, Metadata, ModuleType, H256,
    U256,
};

use crate::{
    h256_to_contract_address, AeternityProvider, FateValue, HyperlaneAeternityError,
};

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

/// Map an integer module type from the Sophia contract to a `ModuleType` enum.
///
/// The Sophia contract returns:
///   0 → Unused, 1 → Routing, 2 → Aggregation, 3 → LegacyMultisig,
///   4 → MerkleRootMultisig, 5 → MessageIdMultisig, 6 → Null, 7 → CcipRead
pub(crate) fn fate_to_module_type(value: FateValue) -> ChainResult<ModuleType> {
    let type_id = match value {
        FateValue::Integer(n) => n.to_u32().ok_or_else(|| {
            HyperlaneAeternityError::ContractCallError("module_type overflow for u32".into())
        })?,
        other => {
            return Err(HyperlaneAeternityError::ContractCallError(format!(
                "expected Integer from module_type(), got {:?}",
                other
            ))
            .into())
        }
    };

    match type_id {
        0 => Ok(ModuleType::Unused),
        1 => Ok(ModuleType::Routing),
        2 => Ok(ModuleType::Aggregation),
        3 => Ok(ModuleType::LegacyMultisig),
        4 => Ok(ModuleType::MerkleRootMultisig),
        5 => Ok(ModuleType::MessageIdMultisig),
        6 => Ok(ModuleType::Null),
        7 => Ok(ModuleType::CcipRead),
        n => Err(HyperlaneAeternityError::ContractCallError(format!(
            "unknown module type: {}",
            n
        ))
        .into()),
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
    /// Returns the module type of the ISM compliant with the corresponding
    /// metadata offchain fetching and onchain formatting standard.
    async fn module_type(&self) -> ChainResult<ModuleType> {
        let result = self
            .provider
            .call_contract(&self.contract_address, "module_type", vec![])
            .await?;

        fate_to_module_type(result)
    }

    /// Dry runs the `verify()` ISM call and returns `Some(gas_estimate)` if the call
    /// succeeds.
    ///
    /// Returns `None` for Aeternity — verification is handled entirely on-chain
    /// during `process()`, so there is no meaningful dry-run gas estimate to return.
    async fn dry_run_verify(
        &self,
        _message: &HyperlaneMessage,
        _metadata: &Metadata,
    ) -> ChainResult<Option<U256>> {
        Ok(None)
    }
}

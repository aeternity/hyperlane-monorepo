//! Interacts with the Aeternity blockchain

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Aeternity application-specific functionality
pub mod application;
/// Checkpoint fraud proof queries
pub mod checkpoint_fraud_proofs;
mod config;
/// Sophia contract source stubs for compiler-based calldata encoding/decoding.
pub mod contracts;
mod error;
mod events;
/// Fraud slashing operations
pub mod fraud_slasher;
/// Aeternity event indexers
pub mod indexer;
mod interchain_gas;
mod ism;
mod mailbox;
mod merkle_tree_hook;
/// Middleware wrappers for ICA and ICQ routers
pub mod middleware;
mod provider;
mod rpc;
mod signer;
mod tx;
mod types;
mod utils;
mod validator_announce;
/// Validator staking status queries
pub mod validator_staking;

pub(crate) use {types::*, utils::*};

pub use {
    checkpoint_fraud_proofs::AeCheckpointFraudProofs,
    config::ConnectionConf,
    error::HyperlaneAeternityError,
    fraud_slasher::AeFraudSlasher,
    interchain_gas::AeInterchainGasPaymaster,
    ism::{
        AeAggregationIsm, AeAmountRoutingIsm, AeIsm, AeMultisigIsm, AeRoutingIsm,
        AeTimelockDomainRoutingIsm,
    },
    mailbox::{AeMailbox, AeTxCalldata},
    merkle_tree_hook::AeMerkleTreeHook,
    provider::{AeternityProvider, FateValue},
    signer::AeSigner,
    validator_announce::AeValidatorAnnounce,
    validator_staking::AeValidatorStaking,
};

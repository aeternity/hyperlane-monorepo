//! Interacts with the Aeternity blockchain

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Aeternity application-specific functionality
pub mod application;
mod config;
/// Sophia contract source stubs for compiler-based calldata encoding/decoding.
pub mod contracts;
mod error;
mod events;
/// Aeternity event indexers
pub mod indexer;
mod interchain_gas;
mod ism;
mod mailbox;
mod merkle_tree_hook;
mod provider;
mod rpc;
mod signer;
mod tx;
mod types;
mod utils;
mod validator_announce;

pub(crate) use {types::*, utils::*};

pub use {
    config::ConnectionConf,
    error::HyperlaneAeternityError,
    interchain_gas::AeInterchainGasPaymaster,
    ism::{AeAggregationIsm, AeIsm, AeMultisigIsm, AeRoutingIsm},
    mailbox::{AeMailbox, AeTxCalldata},
    merkle_tree_hook::AeMerkleTreeHook,
    provider::{AeternityProvider, FateValue},
    signer::AeSigner,
    validator_announce::AeValidatorAnnounce,
};

//! Interacts with the Aeternity blockchain

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod config;
mod error;
mod provider;
mod rpc;
mod types;
mod utils;

pub(crate) use utils::*;

#[allow(unused_imports)]
pub(crate) use types::*;

pub use {
    config::ConnectionConf,
    error::HyperlaneAeternityError,
    provider::{AeSigner, AeternityProvider, FateValue},
    types::{
        ae_address_to_h256, account_address_to_h256, contract_address_to_h256,
        h256_to_account_address, h256_to_ae_address, h256_to_contract_address,
    },
    utils::{blake2b_256, blake2b_hex, decode_ae_hash, encode_ae_hash},
};

//! Sophia contract source stubs for compiler-based calldata encoding/decoding.
//!
//! Each constant contains a minimal, self-contained Sophia contract that
//! mirrors the entrypoint signatures of the deployed Hyperlane contracts.
//! The compiler only needs the type information (function name, parameter types,
//! return type) to encode calldata and decode return values -- the bodies are
//! irrelevant and just satisfy the Sophia compiler's requirement that all
//! entrypoints have implementations.

/// Mailbox contract stub (entrypoints for read calls used by the agent).
pub const MAILBOX_SOURCE: &str = r#"@compiler >= 6

contract interface IInterchainSecurityModule =
    entrypoint module_type : () => int
    entrypoint verify : (bytes(), bytes()) => bool

main contract MailboxStub =
    entrypoint nonce() : int = 0
    entrypoint delivered(id : bytes(32)) : bool = false
    entrypoint default_ism() : option(IInterchainSecurityModule) = None
    entrypoint local_domain() : int = 0
    entrypoint latest_dispatched_id() : bytes(32) = #0000000000000000000000000000000000000000000000000000000000000000
    entrypoint get_recipient_ism(recipient : address) : option(IInterchainSecurityModule) = None
"#;

/// MerkleTreeHook contract stub.
pub const MERKLE_TREE_HOOK_SOURCE: &str = r#"@compiler >= 6

main contract MerkleTreeHookStub =
    record merkle_tree = { branch : map(int, bytes(32)), count : int }
    entrypoint count() : int = 0
    entrypoint root() : bytes(32) = #0000000000000000000000000000000000000000000000000000000000000000
    entrypoint latest_checkpoint() : bytes(32) * int = (#0000000000000000000000000000000000000000000000000000000000000000, 0)
    entrypoint tree() : merkle_tree = { branch = {}, count = 0 }
    entrypoint get_mailbox() : address = Contract.address
    entrypoint local_domain() : int = 0
"#;

/// ValidatorAnnounce contract stub.
pub const VALIDATOR_ANNOUNCE_SOURCE: &str = r#"@compiler >= 6

contract ValidatorAnnounceStub =
    entrypoint get_announced_storage_locations(validators : list(bytes(20))) : list(list(string)) = []
"#;

/// InterchainGasPaymaster contract stub.
pub const IGP_SOURCE: &str = r#"@compiler >= 6

main contract IgpStub =
    entrypoint quote_gas_payment(dest_domain : int, gas_amount : int) : int = 0
    entrypoint sequence() : int = 0
"#;

/// MessageIdMultisigIsm contract stub.
pub const MULTISIG_ISM_SOURCE: &str = r#"@compiler >= 6

main contract MultisigIsmStub =
    entrypoint module_type() : int = 0
    entrypoint validators_and_threshold(message : bytes()) : list(bytes(20)) * int = ([], 0)
"#;

/// DomainRoutingIsm contract stub.
pub const ROUTING_ISM_SOURCE: &str = r#"@compiler >= 6

contract interface IInterchainSecurityModule =
    entrypoint module_type : () => int
    entrypoint verify : (bytes(), bytes()) => bool

main contract RoutingIsmStub =
    entrypoint module_type() : int = 0
    entrypoint route(message : bytes()) : IInterchainSecurityModule = abort("stub")
"#;

/// AggregationIsm contract stub.
pub const AGGREGATION_ISM_SOURCE: &str = r#"@compiler >= 6

main contract AggregationIsmStub =
    entrypoint module_type() : int = 0
    entrypoint modules_and_threshold(message : bytes()) : list(address) * int = ([], 0)
"#;

/// Base ISM stub (for generic ISM instances where only `module_type` is called).
pub const BASE_ISM_SOURCE: &str = r#"@compiler >= 6

main contract IsmStub =
    entrypoint module_type() : int = 0
"#;

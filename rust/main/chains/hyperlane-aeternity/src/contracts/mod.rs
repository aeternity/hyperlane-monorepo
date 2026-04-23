//! ACI-based Sophia contract stubs for compiler-based calldata encoding/decoding.
//!
//! ACI (Aeternity Contract Interface) files are generated from the real
//! deployed Sophia contracts and stored in `abis/*.aci.json`, mirroring
//! how Ethereum/Tron/Starknet store ABI JSON in their chain crates.
//!
//! At runtime, minimal Sophia source stubs are generated from the ACI type
//! information, which is what the Sophia compiler's `/encode-calldata` and
//! `/decode-call-result` endpoints require.

use once_cell::sync::Lazy;
use serde_json::Value;

const MAILBOX_ACI: &str = include_str!("../../abis/Mailbox.aci.json");
const MERKLE_TREE_HOOK_ACI: &str = include_str!("../../abis/MerkleTreeHook.aci.json");
const VALIDATOR_ANNOUNCE_ACI: &str = include_str!("../../abis/ValidatorAnnounce.aci.json");
const IGP_ACI: &str = include_str!("../../abis/InterchainGasPaymaster.aci.json");
const MULTISIG_ISM_ACI: &str = include_str!("../../abis/MessageIdMultisigIsm.aci.json");
const ROUTING_ISM_ACI: &str = include_str!("../../abis/DomainRoutingIsm.aci.json");

/// Mailbox contract stub generated from ACI.
pub static MAILBOX_SOURCE: Lazy<String> = Lazy::new(|| aci_to_sophia(MAILBOX_ACI));
/// MerkleTreeHook contract stub generated from ACI.
pub static MERKLE_TREE_HOOK_SOURCE: Lazy<String> =
    Lazy::new(|| aci_to_sophia(MERKLE_TREE_HOOK_ACI));
/// ValidatorAnnounce contract stub generated from ACI.
pub static VALIDATOR_ANNOUNCE_SOURCE: Lazy<String> =
    Lazy::new(|| aci_to_sophia(VALIDATOR_ANNOUNCE_ACI));
/// InterchainGasPaymaster contract stub generated from ACI.
pub static IGP_SOURCE: Lazy<String> = Lazy::new(|| aci_to_sophia(IGP_ACI));
/// MessageIdMultisigIsm contract stub generated from ACI.
pub static MULTISIG_ISM_SOURCE: Lazy<String> = Lazy::new(|| aci_to_sophia(MULTISIG_ISM_ACI));
/// DomainRoutingIsm contract stub generated from ACI.
pub static ROUTING_ISM_SOURCE: Lazy<String> = Lazy::new(|| aci_to_sophia(ROUTING_ISM_ACI));

/// Generic ISM stub for contracts where only `module_type` is called.
pub static BASE_ISM_SOURCE: Lazy<String> = Lazy::new(|| {
    "@compiler >= 6\n\nmain contract IsmStub =\n    entrypoint module_type() : int = 0\n"
        .to_string()
});

/// Aggregation ISM stub for contracts where `module_type` and
/// `modules_and_threshold` are called.
pub static AGGREGATION_ISM_SOURCE: Lazy<String> = Lazy::new(|| {
    "@compiler >= 6\n\n\
     main contract AggregationIsmStub =\n\
     \x20   entrypoint module_type() : int = 0\n\
     \x20   entrypoint modules_and_threshold(message : bytes()) : list(address) * int = ([], 0)\n"
        .to_string()
});

// ---------------------------------------------------------------------------
// ACI → Sophia stub generation
// ---------------------------------------------------------------------------

/// Convert an ACI JSON string to a minimal Sophia source stub.
///
/// The generated source satisfies the Sophia compiler for calldata
/// encoding/decoding — function bodies are dummy implementations.
fn aci_to_sophia(aci_json: &str) -> String {
    let entries: Vec<Value> = serde_json::from_str(aci_json).expect("invalid ACI JSON");

    // Identify the main contract name so we can strip its prefix from
    // qualified typedef references (e.g. "Mailbox.delivery" → "delivery").
    let main_contract_name: Option<String> = entries.iter().find_map(|e| {
        let c = e.get("contract")?;
        if c.get("kind")?.as_str()? == "contract_main" {
            c.get("name")?.as_str().map(String::from)
        } else {
            None
        }
    });

    let mut out = String::from("@compiler >= 6\n");

    // First pass: emit contract interfaces
    for entry in &entries {
        if let Some(contract) = entry.get("contract") {
            let kind = contract.get("kind").and_then(|k| k.as_str()).unwrap_or("");
            if kind == "contract_interface" {
                out.push('\n');
                emit_interface(&mut out, contract);
            }
        }
    }

    // Second pass: emit namespaces with typedefs (needed for record types)
    for entry in &entries {
        if let Some(ns) = entry.get("namespace") {
            let typedefs = ns.get("typedefs").and_then(|t| t.as_array());
            if let Some(tds) = typedefs {
                if !tds.is_empty() {
                    out.push('\n');
                    emit_namespace(&mut out, ns);
                }
            }
        }
    }

    // Third pass: emit main contract
    for entry in &entries {
        if let Some(contract) = entry.get("contract") {
            let kind = contract.get("kind").and_then(|k| k.as_str()).unwrap_or("");
            if kind == "contract_main" {
                out.push('\n');
                emit_main_contract(&mut out, contract, main_contract_name.as_deref());
            }
        }
    }

    out
}

fn emit_interface(out: &mut String, contract: &Value) {
    let name = contract["name"].as_str().unwrap_or("Unknown");
    out.push_str(&format!("contract interface {name} =\n"));

    if let Some(functions) = contract.get("functions").and_then(|f| f.as_array()) {
        for func in functions {
            let fname = func["name"].as_str().unwrap_or("unknown");
            let args = func.get("arguments").and_then(|a| a.as_array());
            let returns = &func["returns"];

            let arg_types: Vec<String> = args
                .map(|a| {
                    a.iter()
                        .map(|arg| aci_type_to_sophia(&arg["type"], None))
                        .collect()
                })
                .unwrap_or_default();

            let ret_type = aci_type_to_sophia(returns, None);
            let arg_str = if arg_types.is_empty() {
                "()".to_string()
            } else {
                format!("({})", arg_types.join(", "))
            };

            out.push_str(&format!(
                "    entrypoint {fname} : {arg_str} => {ret_type}\n"
            ));
        }
    }
}

fn emit_namespace(out: &mut String, ns: &Value) {
    let name = ns["name"].as_str().unwrap_or("Unknown");
    out.push_str(&format!("namespace {name} =\n"));

    if let Some(typedefs) = ns.get("typedefs").and_then(|t| t.as_array()) {
        for td in typedefs {
            let td_name = td["name"].as_str().unwrap_or("unknown");
            let typedef = &td["typedef"];
            if let Some(fields) = typedef.get("record").and_then(|r| r.as_array()) {
                out.push_str(&format!("    record {td_name} = {{ "));
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|f| {
                        let fn_name = f["name"].as_str().unwrap_or("x");
                        let fn_type = aci_type_to_sophia(&f["type"], None);
                        format!("{fn_name} : {fn_type}")
                    })
                    .collect();
                out.push_str(&field_strs.join(", "));
                out.push_str(" }\n");
            }
        }
    }
}

fn emit_main_contract(out: &mut String, contract: &Value, main_name: Option<&str>) {
    let name = contract["name"].as_str().unwrap_or("Unknown");
    out.push_str(&format!("main contract {name}Stub =\n"));

    // Emit record typedefs
    if let Some(typedefs) = contract.get("typedefs").and_then(|t| t.as_array()) {
        for td in typedefs {
            let td_name = td["name"].as_str().unwrap_or("unknown");
            let typedef = &td["typedef"];
            if let Some(fields) = typedef.get("record").and_then(|r| r.as_array()) {
                out.push_str(&format!("    record {td_name} = {{ "));
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|f| {
                        let fn_name = f["name"].as_str().unwrap_or("x");
                        let fn_type = aci_type_to_sophia(&f["type"], main_name);
                        format!("{fn_name} : {fn_type}")
                    })
                    .collect();
                out.push_str(&field_strs.join(", "));
                out.push_str(" }\n");
            }
        }
    }

    if let Some(functions) = contract.get("functions").and_then(|f| f.as_array()) {
        for func in functions {
            let fname = func["name"].as_str().unwrap_or("unknown");
            if fname == "init" {
                continue;
            }

            let stateful = func
                .get("stateful")
                .and_then(|s| s.as_bool())
                .unwrap_or(false);
            let args = func.get("arguments").and_then(|a| a.as_array());
            let returns = &func["returns"];

            let arg_strs: Vec<String> = args
                .map(|a| {
                    a.iter()
                        .map(|arg| {
                            let aname = arg["name"].as_str().unwrap_or("_");
                            let atype = aci_type_to_sophia(&arg["type"], main_name);
                            format!("{aname} : {atype}")
                        })
                        .collect()
                })
                .unwrap_or_default();

            let ret_type = aci_type_to_sophia(returns, main_name);
            let default = aci_type_default(returns);

            let prefix = if stateful {
                "stateful entrypoint"
            } else {
                "entrypoint"
            };
            let args_str = arg_strs.join(", ");

            out.push_str(&format!(
                "    {prefix} {fname}({args_str}) : {ret_type} = {default}\n"
            ));
        }
    }
}

/// Convert an ACI type JSON value to its Sophia source representation.
///
/// `strip_prefix` is the main contract name whose qualified type
/// references (e.g. `"Mailbox.delivery"`) must be turned into bare names
/// because the stub renames the contract to `<Name>Stub`.  Namespace-
/// qualified types (e.g. `"MerkleLib.merkle_tree"`) are left intact.
fn aci_type_to_sophia(ty: &Value, strip_prefix: Option<&str>) -> String {
    match ty {
        Value::String(s) => match s.as_str() {
            "unit" => "unit".into(),
            other => {
                if let Some(prefix) = strip_prefix {
                    if let Some(local) =
                        other.strip_prefix(prefix).and_then(|r| r.strip_prefix('.'))
                    {
                        return local.to_string();
                    }
                }
                other.to_string()
            }
        },
        Value::Object(map) => {
            if let Some(n) = map.get("bytes") {
                match n {
                    Value::Number(num) => format!("bytes({})", num),
                    _ => "bytes()".into(),
                }
            } else if let Some(Value::Array(items)) = map.get("list") {
                let inner = items
                    .first()
                    .map(|v| aci_type_to_sophia(v, strip_prefix))
                    .unwrap_or("int".into());
                format!("list({inner})")
            } else if let Some(Value::Array(items)) = map.get("option") {
                let inner = items
                    .first()
                    .map(|v| aci_type_to_sophia(v, strip_prefix))
                    .unwrap_or("int".into());
                format!("option({inner})")
            } else if let Some(Value::Array(items)) = map.get("map") {
                let key = items
                    .first()
                    .map(|v| aci_type_to_sophia(v, strip_prefix))
                    .unwrap_or("int".into());
                let val = items
                    .get(1)
                    .map(|v| aci_type_to_sophia(v, strip_prefix))
                    .unwrap_or("int".into());
                format!("map({key}, {val})")
            } else if let Some(Value::Array(items)) = map.get("tuple") {
                if items.is_empty() {
                    "unit".into()
                } else {
                    let parts: Vec<String> = items
                        .iter()
                        .map(|v| aci_type_to_sophia(v, strip_prefix))
                        .collect();
                    parts.join(" * ")
                }
            } else if let Some(Value::Array(fields)) = map.get("record") {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|f| {
                        let name = f["name"].as_str().unwrap_or("x");
                        let ftype = aci_type_to_sophia(&f["type"], strip_prefix);
                        format!("{name} : {ftype}")
                    })
                    .collect();
                format!("{{ {} }}", field_strs.join(", "))
            } else {
                "int".into()
            }
        }
        _ => "int".into(),
    }
}

/// Generate a dummy default value for a given ACI type.
fn aci_type_default(ty: &Value) -> String {
    match ty {
        Value::String(s) => match s.as_str() {
            "int" => "0".into(),
            "bool" => "false".into(),
            "string" => "\"\"".into(),
            "address" => "Contract.address".into(),
            "unit" => "()".into(),
            _ => "abort(\"stub\")".into(),
        },
        Value::Object(map) => {
            if let Some(n) = map.get("bytes") {
                match n {
                    Value::Number(num) => {
                        let size = num.as_u64().unwrap_or(1) as usize;
                        format!("#{}", "00".repeat(size))
                    }
                    _ => "#".into(),
                }
            } else if map.contains_key("list") {
                "[]".into()
            } else if map.contains_key("option") {
                "None".into()
            } else if map.contains_key("map") {
                "{}".into()
            } else if let Some(Value::Array(items)) = map.get("tuple") {
                if items.is_empty() {
                    "()".into()
                } else {
                    let parts: Vec<String> = items.iter().map(aci_type_default).collect();
                    format!("({})", parts.join(", "))
                }
            } else {
                "abort(\"stub\")".into()
            }
        }
        _ => "abort(\"stub\")".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aci_type_to_sophia_primitives() {
        assert_eq!(
            aci_type_to_sophia(&Value::String("int".into()), None),
            "int"
        );
        assert_eq!(
            aci_type_to_sophia(&Value::String("bool".into()), None),
            "bool"
        );
        assert_eq!(
            aci_type_to_sophia(&Value::String("address".into()), None),
            "address"
        );
    }

    #[test]
    fn test_aci_type_to_sophia_bytes() {
        let ty: Value = serde_json::from_str(r#"{"bytes": 32}"#).unwrap();
        assert_eq!(aci_type_to_sophia(&ty, None), "bytes(32)");

        let ty_any: Value = serde_json::from_str(r#"{"bytes": "any"}"#).unwrap();
        assert_eq!(aci_type_to_sophia(&ty_any, None), "bytes()");
    }

    #[test]
    fn test_aci_type_to_sophia_list() {
        let ty: Value = serde_json::from_str(r#"{"list": [{"bytes": 20}]}"#).unwrap();
        assert_eq!(aci_type_to_sophia(&ty, None), "list(bytes(20))");
    }

    #[test]
    fn test_aci_type_to_sophia_tuple() {
        let ty: Value = serde_json::from_str(r#"{"tuple": ["int", "bool"]}"#).unwrap();
        assert_eq!(aci_type_to_sophia(&ty, None), "int * bool");

        let ty_empty: Value = serde_json::from_str(r#"{"tuple": []}"#).unwrap();
        assert_eq!(aci_type_to_sophia(&ty_empty, None), "unit");
    }

    #[test]
    fn test_aci_type_strips_main_contract_prefix() {
        assert_eq!(
            aci_type_to_sophia(&Value::String("Mailbox.delivery".into()), Some("Mailbox")),
            "delivery"
        );
        // Namespace-qualified types should NOT be stripped
        assert_eq!(
            aci_type_to_sophia(
                &Value::String("MerkleLib.merkle_tree".into()),
                Some("Mailbox")
            ),
            "MerkleLib.merkle_tree"
        );
        // Without strip_prefix, everything stays as-is
        assert_eq!(
            aci_type_to_sophia(&Value::String("Mailbox.delivery".into()), None),
            "Mailbox.delivery"
        );
    }

    #[test]
    fn test_aci_type_default_values() {
        assert_eq!(aci_type_default(&Value::String("int".into())), "0");
        assert_eq!(aci_type_default(&Value::String("bool".into())), "false");

        let bytes32: Value = serde_json::from_str(r#"{"bytes": 32}"#).unwrap();
        assert_eq!(
            aci_type_default(&bytes32),
            "#0000000000000000000000000000000000000000000000000000000000000000"
        );

        let list_ty: Value = serde_json::from_str(r#"{"list": ["int"]}"#).unwrap();
        assert_eq!(aci_type_default(&list_ty), "[]");
    }

    #[test]
    fn test_mailbox_aci_generates_valid_stub() {
        let source = aci_to_sophia(MAILBOX_ACI);
        assert!(source.contains("@compiler >= 6"));
        assert!(source.contains("main contract MailboxStub"));
        assert!(source.contains("entrypoint nonce("));
        assert!(source.contains("entrypoint delivered("));
        assert!(source.contains("entrypoint local_domain("));
    }

    #[test]
    fn test_validator_announce_aci_generates_valid_stub() {
        let source = aci_to_sophia(VALIDATOR_ANNOUNCE_ACI);
        assert!(source.contains("main contract ValidatorAnnounceStub"));
        assert!(source.contains("entrypoint announce("));
        assert!(source.contains("entrypoint get_announced_storage_locations("));
    }

    #[test]
    fn test_merkle_tree_hook_aci_generates_valid_stub() {
        let source = aci_to_sophia(MERKLE_TREE_HOOK_ACI);
        assert!(source.contains("main contract MerkleTreeHookStub"));
        assert!(source.contains("entrypoint count("));
        assert!(source.contains("entrypoint root("));
        assert!(source.contains("entrypoint latest_checkpoint("));
    }

    #[test]
    fn test_multisig_ism_aci_generates_valid_stub() {
        let source = aci_to_sophia(MULTISIG_ISM_ACI);
        assert!(source.contains("main contract MessageIdMultisigIsmStub"));
        assert!(source.contains("entrypoint validators_and_threshold("));
    }
}

use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::vec;

use crate::startup::BlockChain;
use anyhow::Context;
use regex::RegexBuilder;
use scout_audit_internal::{DetectorImpl, InkDetector, IntoEnumIterator, SorobanDetector};
use serde_json::{json, Value};

/// This function takes an enum variant of a blockchain (defined in startup.rs) and returns an iterator
/// of the detectors for that blockchain.
/// This is used to centralize the generation of outputs, so that we don't have to repeat the same
/// code for each blockchain.
/// This function looks a bit weird, but if a new blockchain is added, it's just a matter of adding
/// a new arm to the match statement, and the enum in scout_audit_internal/src/detectors.rs that implements
/// DetectorImpl.
/// ```rust
///     BlockChain::NewBlockchain => {
///         Box::new(NewBlockchainDetector::iter().map(|e| Box::new(e) as Box<dyn DetectorImpl>))
///     }
/// ```
pub fn get_chain_enum(bc: BlockChain) -> Box<dyn Iterator<Item = Box<dyn DetectorImpl>>> {
    match bc {
        BlockChain::Soroban => {
            Box::new(SorobanDetector::iter().map(|e| Box::new(e) as Box<dyn DetectorImpl>))
        }
        BlockChain::Ink => {
            Box::new(InkDetector::iter().map(|e| Box::new(e) as Box<dyn DetectorImpl>))
        }
    }
}

pub fn format_into_json(
    scout_output: File,
    internals: File,
    bc: BlockChain,
) -> anyhow::Result<String> {
    let json_errors = jsonify(scout_output, internals, bc)?;
    Ok(serde_json::to_string_pretty(&json_errors)?)
}

fn jsonify(
    scout_output: File,
    internals: File,
    bc: BlockChain,
) -> anyhow::Result<serde_json::Value> {
    let json_errors: serde_json::Value = get_errors_from_output(scout_output, internals, bc)?
        .iter()
        .filter(|(_, (spans, _))| !spans.is_empty())
        .map(|(name, (spans, error))| {
            (
                name,
                json!({
                    "error_msg": error,
                    "spans": spans
                }),
            )
        })
        .collect();

    Ok(json_errors)
}

fn get_errors_from_output(
    mut scout_output: File,
    mut scout_internals: File,
    bc: BlockChain,
) -> anyhow::Result<HashMap<String, (Vec<Value>, String)>> {
    let regex = RegexBuilder::new(r"warning:.*")
        .multi_line(true)
        .case_insensitive(true)
        .build()?;

    let mut stderr_string = String::new();
    std::io::Read::read_to_string(&mut scout_output, &mut stderr_string)?;

    let mut scout_internals_spans: Vec<String> = vec![];

    for line in std::io::BufReader::new(&mut scout_internals).lines() {
        let line = line?;
        let span = line.split('@').collect::<Vec<&str>>()[1];
        scout_internals_spans.push(span.to_string());
    }

    let msg_to_name: HashMap<String, String> = get_chain_enum(bc)
        .map(|e| (e.get_lint_message().to_string(), (*e).to_string()))
        .collect();

    let mut errors: HashMap<String, (Vec<Value>, String)> = get_chain_enum(bc)
        .map(|e| ((*e).to_string(), (vec![], "".to_string())))
        .collect();

    let true_finds = regex
        .find_iter(&stderr_string)
        .map(|e| e.as_str())
        .filter(|e| {
            for err in get_chain_enum(bc).map(|e| e.get_lint_message()) {
                if e.contains(err) {
                    return true;
                }
            }
            false
        })
        .collect::<Vec<&str>>();

    assert!(&true_finds.len() == &scout_internals_spans.len());

    for (i, elem) in true_finds.iter().enumerate() {
        let parts = elem.split('\n').collect::<Vec<&str>>();

        for err in get_chain_enum(bc).map(|e| e.get_lint_message()) {
            if parts[0].contains(err) {
                let name = msg_to_name.get(err).with_context(|| {
                    format!("Error making json: {} not found in the error map", err)
                })?;

                if let Some((spans, error)) = errors.get_mut(name) {
                    spans.push(
                        serde_json::from_str(&scout_internals_spans[i])
                            .expect("Failed parsing span"),
                    );
                    *error = err.to_string();
                }
            }
        }
    }
    Ok(errors)
}

fn serify(
    scout_output: File,
    scout_internals: File,
    bc: BlockChain,
) -> anyhow::Result<serde_json::Value> {
    let errors: HashMap<String, (Vec<Value>, String)> =
        get_errors_from_output(scout_output, scout_internals, bc)?;

    let sarif_output = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0",
        "version": "2.1.0",
        "runs": [
            {
                "tool": {
                    "driver": {
                        "name": env!("CARGO_PKG_NAME"),
                        "version": env!("CARGO_PKG_VERSION"),
                        "rules": get_chain_enum(bc).filter(|e| {
                            errors.contains_key(&e.to_string()) && !errors.get(&e.to_string()).unwrap().0.is_empty()
                        }).map(|e| {
                            json!({
                                "id": e.to_string(),
                                "shortDescription": {
                                    "text": e.get_lint_message()
                                }})

                        }).collect::<Vec<serde_json::Value>>(),
                        "informationUri": "https://coinfabrik.github.io/scout/",
                    }
                },
                "results": build_sarif_results(&errors)?,
            }
        ]
    });
    let json_errors = serde_json::to_value(sarif_output)?;
    Ok(json_errors)
}

pub fn format_into_sarif(
    scout_output: File,
    scout_internals: File,
    bc: BlockChain,
) -> anyhow::Result<String> {
    Ok(serify(scout_output, scout_internals, bc)?.to_string())
}

fn build_sarif_results(
    errors: &HashMap<String, (Vec<Value>, String)>,
) -> anyhow::Result<Vec<serde_json::Value>> {
    let runs: Vec<Value> = errors
        .iter()
        .flat_map(|(name, (spans, msg))| {
            spans.iter().filter_map(move |span| {
                Some(json!({
                    "ruleId": name,
                    "level": "error",
                    "message": {
                        "text": msg
                    },
                    "locations": [span],
                }))
            })
        })
        .collect();

    Ok(runs)
}

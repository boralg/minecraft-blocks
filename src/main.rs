use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::{fs, process};

#[derive(Debug, Serialize)]
struct BlockVariant {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    blockstate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BlockState {
    #[serde(default)]
    variants: BTreeMap<String, Value>,
    #[serde(default)]
    multipart: Vec<MultipartCase>,
}

#[derive(Debug, Deserialize)]
struct MultipartCase {
    #[serde(default)]
    when: Option<Value>,
    apply: Value,
}

fn main() {
    let blockstates_dir = "mc_data/mc_assets/assets/minecraft/blockstates";

    let entries = fs::read_dir(blockstates_dir).unwrap_or_else(|e| {
        eprintln!("Failed to read directory {}: {}", blockstates_dir, e);
        process::exit(1);
    });

    let mut all_variants = Vec::new();

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let block_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap()
            .to_string();

        let content = fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Failed to read {}: {}", path.display(), e);
            process::exit(1);
        });

        let blockstate: BlockState = serde_json::from_str(&content).unwrap_or_else(|e| {
            eprintln!("Failed to parse {}: {}", path.display(), e);
            process::exit(1);
        });

        if !blockstate.variants.is_empty() {
            for variant_key in blockstate.variants.keys() {
                if variant_key.is_empty() {
                    all_variants.push(BlockVariant { // TODO: stone has multiple textures for a single variant
                        name: block_name.clone(),
                        blockstate: None,
                    });
                } else {
                    all_variants.push(BlockVariant {
                        name: block_name.clone(),
                        blockstate: Some(variant_key.clone()),
                    });
                }
            }
        } else if !blockstate.multipart.is_empty() {
            let properties = multipart_properties(&blockstate.multipart);
            let combinations = generate_combinations(&properties);

            for combo in combinations {
                if combo.is_empty() {
                    all_variants.push(BlockVariant {
                        name: block_name.clone(),
                        blockstate: None,
                    });
                } else {
                    all_variants.push(BlockVariant {
                        name: block_name.clone(),
                        blockstate: Some(combo),
                    });
                }
            }
        } else {
            all_variants.push(BlockVariant {
                name: block_name,
                blockstate: None,
            });
        }
    }

    let json_output = serde_json::to_string_pretty(&all_variants).unwrap();
    println!("{}", json_output);
}

fn multipart_properties(multipart: &[MultipartCase]) -> BTreeMap<String, BTreeSet<String>> {
    let mut properties: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for case in multipart {
        if let Some(when) = &case.when {
            properties_from_condition(when, &mut properties);
        }
    }

    for (_key, values) in properties.iter_mut() {
        if values.len() == 1 {
            let val = values.iter().next().unwrap().clone();
            if val == "true" {
                values.insert("false".to_string());
            } else if val == "false" {
                values.insert("true".to_string());
            }
        }
    }

    properties
}

fn properties_from_condition(
    condition: &Value,
    properties: &mut BTreeMap<String, BTreeSet<String>>,
) {
    match condition {
        Value::Object(obj) => {
            for (key, value) in obj {
                if key == "OR" || key == "AND" {
                    if let Value::Array(arr) = value {
                        for item in arr {
                            properties_from_condition(item, properties);
                        }
                    }
                } else {
                    if let Value::String(val_str) = value {
                        let values: Vec<&str> = val_str.split('|').collect();
                        let prop_set = properties.entry(key.clone()).or_insert_with(BTreeSet::new);
                        for v in values {
                            prop_set.insert(v.to_string());
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn generate_combinations(properties: &BTreeMap<String, BTreeSet<String>>) -> Vec<String> {
    if properties.is_empty() {
        return vec![String::new()];
    }

    let prop_vec: Vec<(&String, &BTreeSet<String>)> = properties.iter().collect();
    let mut results = Vec::new();

    fn recurse(
        prop_vec: &[(&String, &BTreeSet<String>)],
        index: usize,
        current: Vec<(String, String)>,
        results: &mut Vec<String>,
    ) {
        if index == prop_vec.len() {
            if current.is_empty() {
                results.push(String::new());
            } else {
                let state_str = current
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(",");
                results.push(state_str);
            }
            return;
        }

        let (prop_name, values) = prop_vec[index];
        for value in values {
            let mut next = current.clone();
            next.push((prop_name.clone(), value.clone()));
            recurse(prop_vec, index + 1, next, results);
        }
    }

    recurse(&prop_vec, 0, Vec::new(), &mut results);
    results
}

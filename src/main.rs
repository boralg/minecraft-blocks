use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
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

#[derive(Debug, Deserialize)]
struct Model {
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    elements: Vec<ModelElement>,
}

#[derive(Debug, Deserialize)]
struct ModelElement {
    from: [f32; 3],
    to: [f32; 3],
}

fn get_all_full_cube_blocks(models_dir: &str) -> HashSet<String> {
    let mut full_cube_blocks = HashSet::new();
    let mut model_cache: HashMap<String, bool> = HashMap::new();

    let entries = fs::read_dir("mc_data/mc_assets/assets/minecraft/blockstates").unwrap();

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

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let blockstate: BlockState = match serde_json::from_str(&content) {
            Ok(bs) => bs,
            Err(_) => continue,
        };

        let mut all_models_full_cube = true;

        if !blockstate.variants.is_empty() {
            for value in blockstate.variants.values() {
                if !is_variant_full_cube(value, models_dir, &mut model_cache) {
                    all_models_full_cube = false;
                    break;
                }
            }
        } else if !blockstate.multipart.is_empty() {
            all_models_full_cube = false;
        }

        if all_models_full_cube {
            full_cube_blocks.insert(block_name);
        }
    }

    full_cube_blocks
}

fn is_variant_full_cube(
    variant_value: &Value,
    models_dir: &str,
    cache: &mut HashMap<String, bool>,
) -> bool {
    match variant_value {
        Value::Object(obj) => {
            if let Some(model_val) = obj.get("model") {
                if let Some(model_path) = model_val.as_str() {
                    return is_model_full_cube(model_path, models_dir, cache);
                }
            }
            false
        }
        Value::Array(arr) => {
            for item in arr {
                if !is_variant_full_cube(item, models_dir, cache) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

fn is_model_full_cube(
    model_path: &str,
    models_dir: &str,
    cache: &mut HashMap<String, bool>,
) -> bool {
    fn elements_is_cube(model: &Model) -> bool {
        for element in &model.elements {
            if element.from == [0.0, 0.0, 0.0] && element.to == [16.0, 16.0, 16.0] {
                return true;
            }
        }

        false
    }

    if let Some(&result) = cache.get(model_path) {
        return result;
    }

    let model_name = model_path
        .strip_prefix("minecraft:block/")
        .or_else(|| model_path.strip_prefix("block/"))
        .unwrap_or(model_path);

    let model_file = format!("{}/{}.json", models_dir, model_name);

    let content = match fs::read_to_string(&model_file) {
        Ok(c) => c,
        Err(_) => {
            cache.insert(model_path.to_string(), false);
            return false;
        }
    };

    let model: Model = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(_) => {
            cache.insert(model_path.to_string(), false);
            return false;
        }
    };

    let has_full_cube = elements_is_cube(&model);

    if has_full_cube {
        cache.insert(model_path.to_string(), true);
        return true;
    }

    if let Some(parent) = &model.parent {
        let parent_is_full = is_model_full_cube(parent, models_dir, cache);
        cache.insert(model_path.to_string(), parent_is_full);
        return parent_is_full;
    }

    cache.insert(model_path.to_string(), false);
    false
}


fn get_all_block_variants(blockstates_dir: &str) -> Vec<BlockVariant> {
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
                    all_variants.push(BlockVariant {
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

    all_variants
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

fn main() {
    let blockstates_dir = "mc_data/mc_assets/assets/minecraft/blockstates";
    let models_dir = "mc_data/mc_assets/assets/minecraft/models/block";

    let all_variants = get_all_block_variants(blockstates_dir);

    let json_output = serde_json::to_string_pretty(&all_variants).unwrap();
    fs::write("blocks.json", &json_output).unwrap_or_else(|e| {
        eprintln!("Failed to write blocks.json: {}", e);
        process::exit(1);
    });
    println!("Saved {} block variants to blocks.json", all_variants.len());

    let full_cube_blocks = get_all_full_cube_blocks(models_dir);

    let full_variants: Vec<BlockVariant> = all_variants
        .into_iter()
        .filter(|v| full_cube_blocks.contains(&v.name))
        .collect();

    let full_json_output = serde_json::to_string_pretty(&full_variants).unwrap();
    fs::write("full_blocks.json", &full_json_output).unwrap_or_else(|e| {
        eprintln!("Failed to write full_blocks.json: {}", e);
        process::exit(1);
    });
    println!(
        "Saved {} full cube block variants to full_blocks.json",
        full_variants.len()
    );
}
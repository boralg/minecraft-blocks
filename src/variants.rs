use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Serialize;

use crate::{
    schema::blockstate::{BlockState, MultipartCase, MultipartCondition, PropertyMatch},
    textures::FaceTextureRefs,
};

#[derive(Clone, Debug, Serialize)]
pub struct BlockVariant {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockstate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub textures: Option<FaceTextureRefs>,
}

pub fn get_all_block_variants(blockstates: &HashMap<String, BlockState>) -> Vec<BlockVariant> {
    let mut all_variants = Vec::new();

    for (block_name, blockstate) in blockstates {
        if !blockstate.variants.is_empty() {
            for variant_key in blockstate.variants.keys() {
                if variant_key.is_empty() {
                    all_variants.push(BlockVariant {
                        name: block_name.clone(),
                        blockstate: None,
                        textures: None,
                    });
                } else {
                    all_variants.push(BlockVariant {
                        name: block_name.clone(),
                        blockstate: Some(variant_key.clone()),
                        textures: None,
                    });
                }
            }
        } else if !blockstate.multipart.is_empty() {
            let properties = multipart_properties(&blockstate.multipart);
            let combinations = multipart_combinations(&properties);

            for combo in combinations {
                if combo.is_empty() {
                    all_variants.push(BlockVariant {
                        name: block_name.clone(),
                        blockstate: None,
                        textures: None,
                    });
                } else {
                    all_variants.push(BlockVariant {
                        name: block_name.clone(),
                        blockstate: Some(combo),
                        textures: None,
                    });
                }
            }
        } else {
            all_variants.push(BlockVariant {
                name: block_name.clone(),
                blockstate: None,
                textures: None,
            });
        }
    }

    all_variants.sort_by(|v1, v2| v1.name.cmp(&v2.name));

    all_variants
}

fn multipart_properties(multipart: &[MultipartCase]) -> BTreeMap<String, BTreeSet<String>> {
    fn add_properties(
        property_match: &PropertyMatch,
        properties: &mut BTreeMap<String, BTreeSet<String>>,
    ) {
        for prop in property_match.properties.keys() {
            let prop_set = properties
                .entry(prop.to_owned())
                .or_insert_with(BTreeSet::new);
            for p in property_match.property_values(prop).unwrap_or(vec![]) {
                prop_set.insert(p.to_owned());
            }
        }
    }

    fn properties_from_condition(
        condition: &MultipartCondition,
        properties: &mut BTreeMap<String, BTreeSet<String>>,
    ) {
        match condition {
            MultipartCondition::Or { or: conds } | MultipartCondition::And { and: conds } => {
                for cond in conds {
                    add_properties(cond, properties);
                }
            }
            MultipartCondition::Properties(property_match) => {
                add_properties(property_match, properties);
            }
        }
    }

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

fn multipart_combinations(properties: &BTreeMap<String, BTreeSet<String>>) -> Vec<String> {
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

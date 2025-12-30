use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BlockState {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub variants: BTreeMap<String, ModelVariant>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub multipart: Vec<MultipartCase>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ModelVariant {
    Single(ModelDefinition),
    Multiple(Vec<ModelDefinition>),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelDefinition {
    pub model: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub x: i32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub y: i32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub z: i32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub uvlock: bool,
    #[serde(default = "default_weight", skip_serializing_if = "is_one")]
    pub weight: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MultipartCase {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<MultipartCondition>,
    pub apply: ModelVariant,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum MultipartCondition {
    Or {
        #[serde(rename = "OR")]
        or: Vec<PropertyMatch>,
    },
    And {
        #[serde(rename = "AND")]
        and: Vec<PropertyMatch>,
    },
    Properties(PropertyMatch),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PropertyMatch {
    #[serde(flatten)]
    pub properties: BTreeMap<String, String>,
}

impl PropertyMatch {
    pub fn property_values(&self, property: &str) -> Option<Vec<&str>> {
        self.properties
            .get(property)
            .map(|v| v.split('|').collect())
    }

    pub fn matches(&self, state: &BTreeMap<String, String>) -> bool {
        for (key, values) in &self.properties {
            if let Some(state_value) = state.get(key) {
                let allowed_values: Vec<&str> = values.split('|').collect();
                if !allowed_values.contains(&state_value.as_str()) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

fn is_zero(value: &i32) -> bool {
    *value == 0
}

fn is_one(value: &i32) -> bool {
    *value == 1
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn default_weight() -> i32 {
    1
}

impl BlockState {
    pub fn is_variants(&self) -> bool {
        !self.variants.is_empty()
    }

    pub fn is_multipart(&self) -> bool {
        !self.multipart.is_empty()
    }
}

impl ModelVariant {
    pub fn models(&self) -> Vec<&ModelDefinition> {
        match self {
            ModelVariant::Single(model) => vec![model],
            ModelVariant::Multiple(models) => models.iter().collect(),
        }
    }
}

pub fn load_all<P: AsRef<Path>>(dir: P) -> io::Result<HashMap<String, BlockState>> {
    let mut blockstates = HashMap::new();

    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let block_name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let content = content
            .replace("minecraft:block/", "")
            .replace("block/", "");

        let blockstate: BlockState = match serde_json::from_str(&content) {
            Ok(bs) => bs,
            Err(_) => continue,
        };

        blockstates.insert(block_name, blockstate);
    }

    Ok(blockstates)
}

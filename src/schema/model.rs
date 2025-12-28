use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Model {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub ambientocclusion: bool,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub display: HashMap<String, DisplayTransform>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub textures: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<Element>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DisplayTransform {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f32; 3]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<[f32; 3]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f32; 3]>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Element {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    pub from: [f32; 3],
    pub to: [f32; 3],

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<ElementRotation>,

    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub shade: bool,

    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub light_emission: i32,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub faces: HashMap<String, Face>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ElementRotation {
    pub origin: [f32; 3],
    pub axis: Axis,
    pub angle: f32,

    #[serde(default, skip_serializing_if = "is_false")]
    pub rescale: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Face {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv: Option<[f32; 4]>,

    pub texture: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cullface: Option<String>,

    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub rotation: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tintindex: Option<i32>,
}

fn default_true() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_zero_i32(value: &i32) -> bool {
    *value == 0
}

pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Model> {
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn load_all<P: AsRef<Path>>(dir: P) -> io::Result<HashMap<String, Model>> {
    let mut models = HashMap::new();

    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let model_name = match path.file_stem().and_then(|s| s.to_str()) {
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

        let model: Model = match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(_) => continue,
        };

        models.insert(model_name, model);
    }

    Ok(models)
}

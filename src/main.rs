mod cubes;
mod schema;
mod textures;
mod variants;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fs, process};

use crate::cubes::{get_all_empty_blocks, get_all_full_cube_blocks};
use crate::schema::{blockstate, model};
use crate::textures::get_block_textures;
use crate::variants::{BlockVariant, get_all_block_variants};

#[derive(Debug, Deserialize)]
struct Model {
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    elements: Vec<ModelElement>,
    #[serde(default)]
    textures: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
struct ModelElement {
    from: [f32; 3],
    to: [f32; 3],
    #[serde(default)]
    faces: HashMap<String, Face>,
}

#[derive(Debug, Deserialize, Clone)]
struct Face {
    texture: String,
}

fn main() {
    let blockstates = blockstate::load_all("mc_data/mc_assets/assets/minecraft/blockstates")
        .expect("Blockstates should be valid");
    let models = model::load_all("mc_data/mc_assets/assets/minecraft/models/block")
        .expect("Block models should be valid");

    let all_variants = get_all_block_variants(&blockstates);

    let json_output = serde_json::to_string_pretty(&all_variants).unwrap();
    fs::write("blocks.json", &json_output).unwrap_or_else(|e| {
        eprintln!("Failed to write blocks.json: {}", e);
        process::exit(1);
    });
    println!("Saved {} block variants to blocks.json", all_variants.len());

    let full_cube_blocks = get_all_full_cube_blocks(&blockstates, &models);

    let full_variants: Vec<BlockVariant> = all_variants
        .into_iter()
        .filter(|v| full_cube_blocks.contains(&v.name))
        .map(|mut v| {
            v.textures = get_block_textures(
                &v.name,
                &v.blockstate.clone().unwrap_or_default(),
                &models,
                &blockstates,
            );
            v
        })
        .collect();

    let json_output = serde_json::to_string_pretty(&full_variants).unwrap();
    fs::write("full_blocks.json", &json_output).unwrap_or_else(|e| {
        eprintln!("Failed to write full_blocks.json: {}", e);
        process::exit(1);
    });
    println!(
        "Saved {} full cube block variants to full_blocks.json",
        full_variants.len()
    );

    let empty_blocks = get_all_empty_blocks(&blockstates, &models);

    let json_output = serde_json::to_string_pretty(&empty_blocks).unwrap();
    fs::write("empty_blocks.json", &json_output).unwrap_or_else(|e| {
        eprintln!("Failed to write empty_blocks.json: {}", e);
        process::exit(1);
    });
    println!(
        "Saved {} empty blocks to empty_blocks.json",
        empty_blocks.len()
    );
}

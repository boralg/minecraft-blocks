mod cubes;
mod palette;
mod schema;
mod textures;
mod variants;

use std::{collections::HashSet, fs, path::Path, process};

use crate::cubes::{get_all_empty_blocks, get_all_full_cube_blocks};
use crate::palette::{BlockTexture, Material, MaterialDisplay, Palette};
use crate::schema::{blockstate, model};
use crate::textures::get_block_textures;
use crate::variants::{BlockVariant, get_all_block_variants};

const MC_DIR: &str = "mc_data/mc_assets/assets/minecraft";

fn main() {
    let mc_dir = Path::new(MC_DIR);

    let output_dir = Path::new("minecraft_palette");
    let textures_dir = output_dir.join("textures");

    fs::create_dir_all(&textures_dir).unwrap_or_else(|e| {
        eprintln!("Failed to create directory structure: {}", e);
        process::exit(1);
    });

    let blockstates =
        blockstate::load_all(mc_dir.join("blockstates")).expect("Blockstates should be valid");
    let models =
        model::load_all(mc_dir.join("models/block")).expect("Block models should be valid");

    let all_variants = get_all_block_variants(&blockstates);

    let json_output = serde_json::to_string_pretty(&all_variants).unwrap();
    let blocks_path = output_dir.join("blocks.json");
    fs::write(&blocks_path, &json_output).unwrap_or_else(|e| {
        eprintln!("Failed to write blocks.json: {}", e);
        process::exit(1);
    });
    println!("Saved {} block variants", all_variants.len());

    let full_cube_blocks = get_all_full_cube_blocks(&blockstates, &models);

    let full_variants: Vec<Material> = all_variants
        .into_iter()
        .filter(|v| full_cube_blocks.contains(&v.name))
        .map(|v| {
            let texture = get_block_textures(
                &v.name,
                &v.blockstate.clone().unwrap_or_default(),
                &models,
                &blockstates,
            );

            let block_id = if let Some(b) = v.blockstate {
                format!("{}#{}", v.name, b)
            } else {
                v.name.to_owned()
            };

            Material {
                block_id,
                display: MaterialDisplay::Texture(texture),
                profile: None,
            }
        })
        .collect();

    let materials: Vec<Material> = vec![];

    let json_output = serde_json::to_string_pretty(&full_variants).unwrap();
    let full_blocks_path = output_dir.join("full_blocks.json");
    fs::write(&full_blocks_path, &json_output).unwrap_or_else(|e| {
        eprintln!("Failed to write full_blocks.json: {}", e);
        process::exit(1);
    });
    println!("Saved {} full cube block variants", full_variants.len(),);

    let empty_blocks = get_all_empty_blocks(&blockstates, &models);

    let json_output = serde_json::to_string_pretty(&empty_blocks).unwrap();
    let empty_blocks_path = output_dir.join("empty_blocks.json");
    fs::write(&empty_blocks_path, &json_output).unwrap_or_else(|e| {
        eprintln!("Failed to write empty_blocks.json: {}", e);
        process::exit(1);
    });
    println!("Saved {} empty blocks", empty_blocks.len());

    copy_textures_from_variants(
        &full_variants,
        &mc_dir.join("textures/block"),
        &textures_dir,
    );
}

fn copy_textures_from_variants(variants: &Vec<Material>, source_dir: &Path, output_dir: &Path) {
    let mut textures = HashSet::new();

    for variant in variants {
        if let MaterialDisplay::Texture(ts) = &variant.display {
            let ts = [&ts.z, &ts.nz, &ts.y, &ts.ny, &ts.x, &ts.nx];

            for t in ts {
                textures.insert(t.path.clone());
            }
        }
    }

    let mut copied = 0;
    let mut failed = 0;

    for t in textures {
        let source_path = source_dir.join(&t).with_extension("png");
        let output_path = output_dir.join(&t).with_extension("png");

        if Path::new(&source_path).exists() {
            if let Err(e) = fs::copy(&source_path, &output_path) {
                eprintln!("Failed to copy {}: {}", source_path.display(), e);
                failed += 1;
            } else {
                copied += 1;
            }
        } else {
            eprintln!("Texture file not found: {}", source_path.display());
            failed += 1;
        }
    }

    println!("Saved {} textures", copied);
    if failed > 0 {
        println!("Failed to copy {} textures", failed);
    }
}

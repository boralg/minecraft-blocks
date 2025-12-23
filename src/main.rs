use std::{fs, process};
use serde_json::Value;

fn main() {
    let blockstates_dir = "mc_data/mc_assets/assets/minecraft/blockstates";

    let entries = fs::read_dir(blockstates_dir).unwrap_or_else(|e| {
        eprintln!("Failed to read directory {}: {}", blockstates_dir, e);
        process::exit(1);
    });

    let mut blocks = Vec::new();

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

        blocks.push(block_name);
    }

    blocks.sort();

    let json_output = serde_json::to_string_pretty(&blocks).unwrap();
    println!("{}", json_output);
}

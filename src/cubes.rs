use std::collections::{HashMap, HashSet};

use crate::schema::{
    blockstate::{BlockState, ModelVariant},
    model::Model,
};

pub fn get_all_empty_blocks(
    blockstates: &HashMap<String, BlockState>,
    models: &HashMap<String, Model>,
) -> HashSet<String> {
    let mut empty_blocks = HashSet::new();

    for (block_name, blockstate) in blockstates {
        if blockstate.is_multipart() || blockstate.variants.len() != 1 {
            continue;
        }

        let (_, v) = blockstate.variants.iter().next().unwrap();
        if v.models().len() != 1 {
            continue;
        }

        let model = models
            .get(&v.models()[0].model)
            .expect("Model should exist");
        if model.elements.is_empty() && model.parent.is_none() && model.textures.is_empty() {
            empty_blocks.insert(block_name.to_owned());
        }
    }

    empty_blocks
}

pub fn get_all_full_cube_blocks(
    blockstates: &HashMap<String, BlockState>,
    models: &HashMap<String, Model>,
) -> HashSet<String> {
    let mut full_cube_blocks = HashSet::new();

    for (block_name, blockstate) in blockstates {
        let mut all_models_full_cube = true;

        if !blockstate.variants.is_empty() {
            for value in blockstate.variants.values() {
                if !is_variant_full_cube(value, models) {
                    all_models_full_cube = false;
                    break;
                }
            }
        } else if !blockstate.multipart.is_empty() {
            all_models_full_cube = false;
        }

        if all_models_full_cube {
            full_cube_blocks.insert(block_name.to_owned());
        }
    }

    full_cube_blocks
}

fn is_variant_full_cube(variant: &ModelVariant, models: &HashMap<String, Model>) -> bool {
    match variant {
        ModelVariant::Single(v) => is_model_full_cube(&v.model, models),
        ModelVariant::Multiple(vs) => vs.iter().all(|v| is_model_full_cube(&v.model, models)),
    }
}

fn is_model_full_cube(model_name: &str, models: &HashMap<String, Model>) -> bool {
    let model = models.get(model_name).expect("Model should exist");

    let is_full_cube = !model.elements.is_empty()
        && model
            .elements
            .iter()
            .all(|e| e.from == [0.0, 0.0, 0.0] && e.to == [16.0, 16.0, 16.0]);

    if is_full_cube {
        return true;
    }

    if let Some(parent) = &model.parent {
        let parent_is_full = is_model_full_cube(parent, models);
        return parent_is_full;
    }

    false
}

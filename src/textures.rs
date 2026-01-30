use std::collections::HashMap;

use serde::Serialize;

use crate::{
    palette::{BlockTexture, FaceTexture, Rotation},
    schema::{
        blockstate::BlockState,
        model::{Element, Model},
    },
};

#[derive(Debug, Serialize, Clone)]
pub struct FaceTextureRefs {
    pub x: String,
    pub nx: String,
    pub y: String,
    pub ny: String,
    pub z: String,
    pub nz: String,
}

impl BlockTexture {
    fn to_face_texture_refs(&self) -> FaceTextureRefs {
        FaceTextureRefs {
            x: self.x.to_string(),
            nx: self.nx.to_string(),
            y: self.y.to_string(),
            ny: self.ny.to_string(),
            z: self.z.to_string(),
            nz: self.nz.to_string(),
        }
    }
}

pub fn get_block_textures(
    block_name: &str,
    blockstate_key: &str,
    models: &HashMap<String, Model>,
    blockstates: &HashMap<String, BlockState>,
) -> BlockTexture {
    let blockstate = blockstates
        .get(block_name)
        .expect("Blockstate should exist");

    let m = blockstate
        .variants
        .get(blockstate_key)
        .expect("Blockstate key should exist")
        .models()[0]; // TODO: multiple textures per block e.g. stone

    let model = models.get(&m.model).expect("Model should exist");
    let cube_element = full_cube_element(model, models);

    let [down, up, north, south, west, east] = ["down", "up", "north", "south", "west", "east"]
        .map(|f| {
            resolve_texture_variable(
                &cube_element
                    .faces
                    .get(f)
                    .expect(&format!("Face texture of {} should be present", f))
                    .texture,
                &model.textures,
                models,
                &model.parent,
            )
        });

    let mut block_texture = BlockTexture {
        x: FaceTexture::new(east),
        nx: FaceTexture::new(west),
        y: FaceTexture::new(up),
        ny: FaceTexture::new(down),
        z: FaceTexture::new(north),
        nz: FaceTexture::new(south),
    };

    block_texture = block_texture.rotate_x(Rotation::from_degrees(m.x).unwrap());
    block_texture = block_texture.rotate_y(Rotation::from_degrees(m.y).unwrap());
    block_texture = block_texture.rotate_z(Rotation::from_degrees(m.z).unwrap());

    block_texture
}

fn resolve_texture_variable(
    var_name: &str,
    textures: &HashMap<String, String>,
    models: &HashMap<String, Model>,
    parent: &Option<String>,
) -> String {
    let mut textures = textures.clone();
    let mut parent = parent;

    while let Some(parent_name) = parent {
        let par = models.get(parent_name).expect("Model should exist");

        for (k, v) in par.textures.clone() {
            if !textures.contains_key(&k) {
                textures.insert(k, v);
            }
        }

        parent = &par.parent;
    }

    let mut var_name = var_name.strip_prefix('#').unwrap_or(var_name);

    while let Some(value) = textures.get(var_name) {
        if value.starts_with('#') {
            var_name = value.strip_prefix('#').unwrap();
        } else {
            return value.clone();
        }
    }

    panic!("Texture variable {var_name} should be resolvable")
}

fn full_cube_element(model: &Model, models: &HashMap<String, Model>) -> Element {
    if let Some(cube) = model
        .elements
        .iter()
        .find(|e| e.from == [0.0, 0.0, 0.0] && e.to == [16.0, 16.0, 16.0])
    {
        return cube.clone();
    }

    if let Some(parent_name) = &model.parent {
        let parent_model = models.get(parent_name).expect("Model should exist");

        return full_cube_element(&parent_model, models);
    }

    panic!("Model should have a full cube element");
}

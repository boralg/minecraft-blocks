use std::{collections::HashMap, fs};

use serde::Serialize;

use crate::{
    schema::{
        blockstate::{self, BlockState},
        model::{Element, Model},
    },
    textures,
};

#[derive(Debug, Serialize, Clone)]
pub struct FaceTextureRefs {
    down: String,
    up: String,
    north: String,
    south: String,
    west: String,
    east: String,
}

#[derive(Debug, Clone)]
pub struct Texture {
    path: String,
    rotation: i32,
}

impl Texture {
    fn new(path: String) -> Self {
        Self { path, rotation: 0 }
    }

    fn add_rotation(&self, degrees: i32) -> Self {
        Self {
            path: self.path.clone(),
            rotation: (self.rotation + degrees) % 360,
        }
    }

    fn to_string(&self) -> String {
        if self.rotation == 0 {
            self.path.clone()
        } else {
            format!("{}#{}", self.path, self.rotation)
        }
    }
}

#[derive(Debug, Clone)]
pub struct FaceTextures {
    down: Texture,
    up: Texture,
    north: Texture,
    south: Texture,
    west: Texture,
    east: Texture,
}

impl FaceTextures {
    fn rotate_x(self, degrees: i32) -> Self {
        if degrees == 0 {
            return self;
        }
        Self {
            down: self.north.add_rotation(0),
            up: self.south.add_rotation(0),
            north: self.up.add_rotation(0),
            south: self.down.add_rotation(0),
            west: self.west.add_rotation(360 - degrees),
            east: self.east.add_rotation(degrees),
        }
    }

    fn rotate_y(self, degrees: i32) -> Self {
        if degrees == 0 {
            return self;
        }
        Self {
            down: self.down.add_rotation(360 - degrees),
            up: self.up.add_rotation(degrees),
            north: self.east.add_rotation(0),
            south: self.west.add_rotation(0),
            west: self.north.add_rotation(0),
            east: self.south.add_rotation(0),
        }
    }

    fn rotate_z(self, degrees: i32) -> Self {
        if degrees == 0 {
            return self;
        }
        Self {
            down: self.west.add_rotation(0),
            up: self.east.add_rotation(0),
            north: self.north.add_rotation(degrees),
            south: self.south.add_rotation(360 - degrees),
            west: self.up.add_rotation(0),
            east: self.down.add_rotation(0),
        }
    }

    fn to_face_texture_refs(&self) -> FaceTextureRefs {
        FaceTextureRefs {
            down: self.down.to_string(),
            up: self.up.to_string(),
            north: self.north.to_string(),
            south: self.south.to_string(),
            west: self.west.to_string(),
            east: self.east.to_string(),
        }
    }
}

pub fn get_block_textures(
    block_name: &str,
    blockstate_key: &str,
    models: &HashMap<String, Model>,
    blockstates: &HashMap<String, BlockState>,
) -> Option<FaceTextureRefs> {
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

    let mut face_textures = FaceTextures {
        down: Texture::new(down),
        up: Texture::new(up),
        north: Texture::new(north),
        south: Texture::new(south),
        west: Texture::new(west),
        east: Texture::new(east),
    };

    face_textures = face_textures.rotate_x(m.x);
    face_textures = face_textures.rotate_y(m.y);
    face_textures = face_textures.rotate_z(m.z);

    Some(face_textures.to_face_texture_refs())
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

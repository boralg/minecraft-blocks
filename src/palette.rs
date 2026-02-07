use std::{
    collections::{BTreeSet, HashSet},
    fs,
    path::Path,
    time::Duration,
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub struct Palette {
    pub name: String,
    pub id: String,
    pub materials: IndexMap<String, Material>,
    pub groups: IndexMap<String, Group>,
    pub variant_sets: IndexMap<String, VariantSet>,
}

impl Palette {
    pub fn deserialize_from_dir<P: AsRef<Path>>(palette_dir: P) -> Result<Self, String> {
        let palette_dir = palette_dir.as_ref();

        let id = palette_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid palette directory name")?
            .to_string();

        let name = id.clone();

        // TODO: this doesn't error on duplicate keys (serde default behavior) and I hate every solution I've found (including forking indexmap)
        let materials_json = fs::read_to_string(&palette_dir.join("materials.json"))
            .map_err(|e| format!("Failed to read materials.json: {}", e))?;
        let materials = serde_json::from_str(&materials_json)
            .map_err(|e| format!("Failed to parse materials.json: {}", e))?;

        let groups_json = fs::read_to_string(&palette_dir.join("groups.json"))
            .map_err(|e| format!("Failed to read groups.json: {}", e))?;
        let groups = serde_json::from_str(&groups_json)
            .map_err(|e| format!("Failed to parse groups.json: {}", e))?;

        let variant_sets_json = fs::read_to_string(&palette_dir.join("variant_sets.json"))
            .map_err(|e| format!("Failed to read variant_sets.json: {}", e))?;
        let variant_sets = serde_json::from_str(&variant_sets_json)
            .map_err(|e| format!("Failed to parse variant_sets.json: {}", e))?;

        let palette = Palette {
            name,
            id,
            materials,
            groups,
            variant_sets,
        };

        let textures_dir = palette_dir.join("textures");
        if textures_dir.exists() {
            let mut texture_paths = HashSet::new();
            let mut missing_textures = Vec::new();

            for (_, material) in &palette.materials {
                palette.visit_material(&material.display, &mut |path| {
                    texture_paths.insert(path.to_string());
                });
            }

            for texture_path in texture_paths {
                let texture_file = textures_dir.join(format!("{}.png", texture_path));
                if !texture_file.exists() {
                    missing_textures.push(texture_path);
                }
            }

            if !missing_textures.is_empty() {
                eprintln!("Warning: Missing textures: {}", missing_textures.join(", "));
            }
        } else {
            eprintln!("Warning: Textures directory not found");
        }

        Ok(palette)
    }

    pub fn serialize_to_dir<P: AsRef<Path>>(
        &self,
        output_dir: P,
        textures_dir: P,
    ) -> Result<(), String> {
        let palette_dir = output_dir.as_ref().join(&self.id);
        fs::create_dir_all(&palette_dir)
            .map_err(|e| format!("Failed to create palette directory: {}", e))?;

        {
            let materials_json = serde_json::to_string_pretty(&self.materials)
                .map_err(|e| format!("Failed to serialize materials: {}", e))?;
            fs::write(palette_dir.join("materials.json"), materials_json)
                .map_err(|e| format!("Failed to write materials.json: {}", e))?;

            let groups_json = serde_json::to_string_pretty(&self.groups)
                .map_err(|e| format!("Failed to serialize groups: {}", e))?;
            fs::write(palette_dir.join("groups.json"), groups_json)
                .map_err(|e| format!("Failed to write groups.json: {}", e))?;

            let variant_sets_json = serde_json::to_string_pretty(&self.variant_sets)
                .map_err(|e| format!("Failed to serialize variant sets: {}", e))?;
            fs::write(palette_dir.join("variant_sets.json"), variant_sets_json)
                .map_err(|e| format!("Failed to write variant_sets.json: {}", e))?;
        }

        let output_textures_dir = palette_dir.join("textures");
        fs::create_dir_all(&output_textures_dir)
            .map_err(|e| format!("Failed to create textures directory: {}", e))?;

        {
            let mut texture_paths = HashSet::new();
            let mut missing_textures = Vec::new();

            for (_, material) in &self.materials {
                self.visit_material(&material.display, &mut |path| {
                    let src = textures_dir.as_ref().join(format!("{}.png", path));
                    if src.exists() {
                        texture_paths.insert(path.to_string());
                    } else {
                        missing_textures.push(path.to_string());
                    }
                });
            }

            if !missing_textures.is_empty() {
                return Err(format!("Missing textures: {}", missing_textures.join(", ")));
            }

            for texture_path in texture_paths {
                let src = textures_dir.as_ref().join(format!("{}.png", texture_path));
                let dst = output_textures_dir.join(format!("{}.png", texture_path));

                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create texture subdirectory: {}", e))?;
                }

                fs::copy(&src, &dst)
                    .map_err(|e| format!("Failed to copy texture {}: {}", texture_path, e))?;
            }
        }

        Ok(())
    }

    fn visit_material<F>(&self, display: &MaterialDisplay, visitor: &mut F)
    where
        F: FnMut(&str),
    {
        match display {
            MaterialDisplay::Texture(tex) => {
                self.visit_block_texture_paths(tex, visitor);
            }
            MaterialDisplay::TextureAnimation { frames, .. } => {
                for frame in frames {
                    self.visit_block_texture_paths(frame, visitor);
                }
            }
            MaterialDisplay::Volume(vol) => {
                visitor(&vol.path);
            }
            MaterialDisplay::VolumeAnimation { frames, .. } => {
                for frame in frames {
                    visitor(&frame.path);
                }
            }
        }
    }

    fn visit_block_texture_paths<F>(&self, tex: &BlockTexture, visitor: &mut F)
    where
        F: FnMut(&str),
    {
        visitor(&tex.x.path);
        visitor(&tex.nx.path);
        visitor(&tex.y.path);
        visitor(&tex.ny.path);
        visitor(&tex.z.path);
        visitor(&tex.nz.path);
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Material {
    pub display: MaterialDisplay,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub profile: Option<MaterialProfile>,
}

// TODO: default
#[derive(Clone, Serialize, Deserialize)]
pub struct MaterialProfile {
    pub light_color: Color, // a = luminosity
    pub opaque_bloom: Color,
    pub transparent_bloom: Color,
    pub opaque_reflect: f32,
    pub transparent_reflect: f32,
    pub transparent_refract: f32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaterialDisplay {
    Texture(BlockTexture),
    TextureAnimation {
        frames: Vec<BlockTexture>,
        delay: Duration,
    },
    Volume(BlockVolume),
    VolumeAnimation {
        frames: Vec<BlockVolume>,
        delay: Duration,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockTexture {
    #[serde(
        serialize_with = "serialize_face_string",
        deserialize_with = "deserialize_face_string"
    )]
    pub x: FaceTexture,
    #[serde(
        serialize_with = "serialize_face_string",
        deserialize_with = "deserialize_face_string"
    )]
    pub nx: FaceTexture,
    #[serde(
        serialize_with = "serialize_face_string",
        deserialize_with = "deserialize_face_string"
    )]
    pub y: FaceTexture,
    #[serde(
        serialize_with = "serialize_face_string",
        deserialize_with = "deserialize_face_string"
    )]
    pub ny: FaceTexture,
    #[serde(
        serialize_with = "serialize_face_string",
        deserialize_with = "deserialize_face_string"
    )]
    pub z: FaceTexture,
    #[serde(
        serialize_with = "serialize_face_string",
        deserialize_with = "deserialize_face_string"
    )]
    pub nz: FaceTexture,
}

fn serialize_face_string<S>(face: &FaceTexture, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&face.to_string())
}

fn deserialize_face_string<'de, D>(d: D) -> Result<FaceTexture, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    parse_face_texture(&s).map_err(serde::de::Error::custom)
}

fn parse_face_texture(s: &str) -> Result<FaceTexture, String> {
    let (path, tags) = if let Some((p, t)) = s.split_once('#') {
        (p.to_string(), Some(t))
    } else {
        (s.to_string(), None)
    };

    let mut rotation = Rotation::CCW0;
    let mut flip_x = false;
    let mut flip_y = false;

    if let Some(tags) = tags {
        for tag in tags.split(default_variant_tags::SEP) {
            if tag.is_empty() {
                continue;
            }
            if let Some(deg_str) = tag.strip_prefix(default_variant_tags::ROT_TEX) {
                let deg: i32 = deg_str
                    .parse()
                    .map_err(|e| format!("Invalid rotation: {}", e))?;
                rotation = Rotation::from_degrees(deg)
                    .ok_or_else(|| format!("Invalid rotation degrees: {}", deg))?;
            } else if tag == default_variant_tags::FLIP_X_TEX {
                flip_x = true;
            } else if tag == default_variant_tags::FLIP_Y_TEX {
                flip_y = true;
            }
        }
    }

    Ok(FaceTexture {
        path,
        rotation,
        flip_x,
        flip_y,
    })
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FaceTexture {
    pub path: String,
    #[serde(skip_serializing_if = "is_rotation_zero", default)]
    pub rotation: Rotation,
    #[serde(skip_serializing_if = "is_false", default)]
    pub flip_x: bool,
    #[serde(skip_serializing_if = "is_false", default)]
    pub flip_y: bool,
}

impl FaceTexture {
    pub fn new(path: String) -> Self {
        Self {
            path,
            rotation: Rotation::CCW0,
            flip_x: false,
            flip_y: false,
        }
    }

    pub fn add_rotation(&self, rotation: Rotation) -> Self {
        Self {
            path: self.path.clone(),
            rotation: self.rotation.add(rotation),
            flip_x: false,
            flip_y: false,
        }
    }

    pub fn to_string(&self) -> String {
        let mut suffix = String::new();
        if self.rotation != Rotation::CCW0 {
            suffix.push(default_variant_tags::SEP);
            suffix.push_str(default_variant_tags::ROT_TEX);
            suffix.push_str(&self.rotation.degrees().to_string());
        }
        if self.flip_x {
            suffix.push(default_variant_tags::SEP);
            suffix.push_str(default_variant_tags::FLIP_X_TEX);
        }
        if self.flip_y {
            suffix.push(default_variant_tags::SEP);
            suffix.push_str(default_variant_tags::FLIP_Y_TEX);
        }

        if suffix.is_empty() {
            self.path.clone()
        } else {
            format!("{}{}", self.path, suffix)
        }
    }
}

impl BlockTexture {
    pub fn rotate_x(self, rot: Rotation) -> Self {
        let mut t = self;

        for _ in 0..(rot.degrees() / 90) {
            t = Self {
                ny: t.z,
                y: t.nz,
                z: t.y,
                nz: t.ny,
                nx: t.nx.add_rotation(Rotation::CCW270),
                x: t.x.add_rotation(Rotation::CCW90),
            };
        }

        t
    }

    pub fn rotate_y(self, rot: Rotation) -> Self {
        let mut t = self;

        for _ in 0..(rot.degrees() / 90) {
            t = Self {
                ny: t.ny.add_rotation(Rotation::CCW270),
                y: t.y.add_rotation(Rotation::CCW90),
                z: t.x,
                nz: t.nx,
                nx: t.z,
                x: t.nz,
            };
        }

        t
    }

    pub fn rotate_z(self, rot: Rotation) -> Self {
        let mut t = self;

        for _ in 0..(rot.degrees() / 90) {
            t = Self {
                ny: t.nx,
                y: t.x,
                z: t.z.add_rotation(Rotation::CCW90),
                nz: t.nz.add_rotation(Rotation::CCW270),
                nx: t.y,
                x: t.ny,
            };
        }

        t
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockVolume {
    pub path: String,
    #[serde(skip_serializing_if = "is_rotation_zero", default)]
    pub rotation_x: Rotation,
    #[serde(skip_serializing_if = "is_rotation_zero", default)]
    pub rotation_y: Rotation,
    #[serde(skip_serializing_if = "is_rotation_zero", default)]
    pub rotation_z: Rotation,
    #[serde(skip_serializing_if = "is_false", default)]
    pub flip_x: bool,
    #[serde(skip_serializing_if = "is_false", default)]
    pub flip_y: bool,
    #[serde(skip_serializing_if = "is_false", default)]
    pub flip_z: bool,
}

fn is_rotation_zero(r: &Rotation) -> bool {
    *r == Rotation::CCW0
}

fn is_false(b: &bool) -> bool {
    !b
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(into = "i32", try_from = "i32")]
pub enum Rotation {
    #[default]
    CCW0,
    CCW90,
    CCW180,
    CCW270,
}

impl Rotation {
    pub fn degrees(self) -> i32 {
        match self {
            Rotation::CCW0 => 0,
            Rotation::CCW90 => 90,
            Rotation::CCW180 => 180,
            Rotation::CCW270 => 270,
        }
    }

    pub fn from_degrees(deg: i32) -> Option<Self> {
        match deg {
            0 => Some(Rotation::CCW0),
            90 => Some(Rotation::CCW90),
            180 => Some(Rotation::CCW180),
            270 => Some(Rotation::CCW270),
            _ => None,
        }
    }

    pub fn add(self, rotation: Self) -> Self {
        Self::from_degrees((self.degrees() + rotation.degrees()) % 360).unwrap()
    }
}

impl From<Rotation> for i32 {
    fn from(rotation: Rotation) -> Self {
        rotation.degrees()
    }
}

impl TryFrom<i32> for Rotation {
    type Error = String;

    fn try_from(deg: i32) -> Result<Self, Self::Error> {
        Self::from_degrees(deg).ok_or_else(|| format!("Invalid rotation degrees: {}", deg))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Group {
    block_ids: BlockIds,
    rule: GroupRule,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockIds {
    Blocks(BTreeSet<String>),
    VariantSet(String),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupRule {
    RandomChoice,
    Custom(serde_json::Value),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VariantSet {
    input_block_ids: BlockIds,
    rotations: Vec<Rotation>,
    flip_x: bool,
    flip_y: bool,
    flip_z: bool,
    tints: Vec<Color>,
    custom: Option<serde_json::Value>,
}

mod default_variant_tags {
    pub const SEP: char = '#';

    pub const ROT_X: &str = "rx=";
    pub const ROT_Y: &str = "ry=";
    pub const ROT_Z: &str = "rz=";
    pub const FLIP_X: &str = "fx";
    pub const FLIP_Y: &str = "fy";
    pub const FLIP_Z: &str = "fz";
    pub const TINT: &str = "tint=";

    pub const ROT_TEX: &str = "r=";
    pub const FLIP_X_TEX: &str = "fx";
    pub const FLIP_Y_TEX: &str = "fy";
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<Color> for String {
    fn from(color: Color) -> Self {
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            color.r, color.g, color.b, color.a
        )
    }
}

impl TryFrom<String> for Color {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let s = s.strip_prefix('#').ok_or("Color must start with #")?;
        if s.len() != 8 {
            return Err(format!("Color must be 8 hex digits, got {}", s.len()));
        }

        let hex = u32::from_str_radix(s, 16).map_err(|e| e.to_string())?;

        Ok(Color {
            r: (hex >> 24) as u8,
            g: (hex >> 16) as u8,
            b: (hex >> 8) as u8,
            a: hex as u8,
        })
    }
}

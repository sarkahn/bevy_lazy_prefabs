use std::ops::Range;

use bevy::{asset::{AssetLoader, LoadedAsset}, prelude::*, reflect::{List, TypeUuid}};
use serde::Deserialize;

use crate::registry::PrefabRegistry;
use crate::dynamic_cast::*;

#[derive(Debug,Deserialize)]
pub enum FieldValue {
    Int(i32),
    Float(f32),
    String(String),
    Char(char),
    Vec(Vec<FieldValue>),
    Bool(bool),
    Range(Range<i32>),
    Struct,
}

impl FieldValue {
    pub fn as_int(&self) -> Option<i32> {
        match self {
            FieldValue::Int(i) => Some(*i),
            _ => None
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            FieldValue::Float(f) => Some(*f),
            _ => None
        }
    }
    
    pub fn as_string(&self) -> Option<&str> {
        match self {
            FieldValue::String(s) => Some(&s),
            _ => None,
        }
    }

    pub fn as_range(&self) -> Option<&Range<i32>> {
        match self {
            FieldValue::Range(r) => Some(r),
            _ => None
        }
    }

    pub fn as_char(&self) -> Option<char> {
        match self {
            FieldValue::Char(c) => Some(*c),
            _ => None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FieldValue::Bool(b) => Some(*b),
            _ => None
        }
    }

    pub fn as_vec(&self) -> Option<&Vec<FieldValue>> {
        match self {
            FieldValue::Vec(v) => Some(v),
            _ => None
        }
    }
}
impl PartialEq for FieldValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(l0), Self::Int(r0)) => l0 == r0,
            (Self::Float(l0), Self::Float(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Char(l0), Self::Char(r0)) => l0 == r0,
            (Self::Vec(l0), Self::Vec(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Range(l0), Self::Range(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for FieldValue {
    
}

#[derive(Debug)]
pub struct PrefabComponentField {
    pub name: String,
    pub value: Box<dyn Reflect>,
}

#[derive(Debug)]
pub struct PrefabComponent {
    pub name: String,
    pub reflect: Box<dyn Reflect>,
}

#[derive(Debug, TypeUuid)]
#[uuid = "289f0b4a-2b90-49d2-af63-61ad2fec867c"]
pub struct Prefab {
    pub name: Option<String>,
    pub components: Vec<PrefabComponent>,
}

impl Prefab {
    pub fn component(&self, name: &str) -> Option<&PrefabComponent> {
        self.components.iter().find(|c| c.name == name)
    }
}

impl Default for Prefab {
    fn default() -> Self {
        Self { 
            name: Default::default(), 
            components: Vec::new(),
        }
    }
}

// impl AssetLoader for Prefab {
//     fn load<'a>(
//         &'a self,
//         bytes: &'a [u8],
//         load_context: &'a mut bevy::asset::LoadContext,
//     ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
//         Box::pin( async move {
//             let prefab_string = std::str::from_utf8(bytes)?;
//             let prefab = Prefab {
//                 //serialized: prefab_string.to_string(),
//                 ..Default::default()
//             };
//             load_context.set_default_asset(LoadedAsset::new(prefab));
            
//             Ok(())
//         })
//     }

//     fn extensions(&self) -> &[&str] {
//         &["prefab"]
//     }
// }

// fn asset_load_system(
//     registry: Res<PrefabRegistry>,
// ) {

// }

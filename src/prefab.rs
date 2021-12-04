use std::ops::Range;

use bevy::{asset::{AssetLoader, LoadedAsset}, prelude::*, reflect::{List, TypeUuid, DynamicStruct, DynamicTupleStruct, DynamicTuple}};
use serde::Deserialize;

use crate::registry::{PrefabRegistry, TypeInfo, ReflectType};
use crate::dynamic_cast::*;

/// A name/value pair representing a field on a type
#[derive(Debug)]
pub struct ReflectField {
    pub name: String,
    pub value: Box<dyn Reflect>,
}

impl From<ReflectField> for (String, Box<dyn Reflect>) {
    fn from(field: ReflectField) -> Self {
        (field.name, field.value)
    }
}

#[derive(Debug)]
pub struct PrefabComponent {
    name: String,
    dynamic_value: Box<dyn Reflect>,
}

impl From<PrefabComponent> for ReflectField {
    fn from(comp: PrefabComponent) -> Self {
        ReflectField {
            name: comp.name,
            value: comp.dynamic_value,
        }
    }
}

impl PrefabComponent {
    pub fn from_type(type_info: &TypeInfo) -> Self {
        type_info.into()
    }

    pub fn new(name: &str, root: Box<dyn Reflect>) -> Self {
        PrefabComponent {
            name: name.to_string(),
            dynamic_value: root
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn root(&self) -> &Box<dyn Reflect> {
        &self.dynamic_value
    }

    pub fn root_mut(&mut self) -> &mut Box<dyn Reflect> {
        &mut self.dynamic_value
    }
}

impl Clone for PrefabComponent {
    fn clone(&self) -> Self {
        Self { 
            name: self.name.clone(), 
            dynamic_value: self.dynamic_value.clone_value() 
        }
    }
}

impl From<&TypeInfo> for PrefabComponent {
    fn from(t: &TypeInfo) -> Self {
        PrefabComponent {
            name: t.type_name.to_string(),
            dynamic_value: match t.reflect_type {
                ReflectType::Struct => Box::new(DynamicStruct::default()),
                ReflectType::TupleStruct => Box::new(DynamicTupleStruct::default()),
                ReflectType::Tuple => Box::new(DynamicTuple::default()),
                ReflectType::List => todo!(),
                ReflectType::Map => todo!(),
                ReflectType::Value => todo!(),
            }
        }
    }
}

#[derive(Debug, TypeUuid, Clone)]
#[uuid = "289f0b4a-2b90-49d2-af63-61ad2fec867c"]
pub struct Prefab {
    name: Option<String>,
    components: Vec<PrefabComponent>,
}

impl Prefab {
    pub fn new(name: Option<String>, components: Vec<PrefabComponent>) -> Self {
        Prefab {
            name,
            components
        }
    }

    pub fn name(&self) -> Option<&str> {
        match &self.name {
            Some(name) => Some(name.as_str()),
            None => None
        }
    }

    pub fn component(&self, name: &str) -> Option<&PrefabComponent> {
        self.components.iter().find(|c| c.name == name)
    }

    pub fn components(&self) -> &Vec<PrefabComponent> {
        &self.components
    }

    pub fn component_from_index(&self, index: usize) -> &PrefabComponent {
        &self.components[index]
    }
}

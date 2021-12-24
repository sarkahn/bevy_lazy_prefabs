use std::{any::TypeId, sync::Arc};

use bevy::{prelude::*, reflect::{DynamicStruct, TypeUuid}};

use derivative::*;

use crate::{commands::PrefabCommand, PrefabProcessor};

#[derive(Derivative, TypeUuid)]
#[derivative(Debug)]
#[uuid = "6ea14da5-6bf8-3ea1-9886-1d7bf6c17d2f"]
pub struct Prefab {
    name: Option<String>,
    #[derivative(Debug = "ignore")]
    commands: Vec<PrefabCommand>,
}

impl Prefab {
    pub(crate) fn new(name: Option<String>, commands: Vec<PrefabCommand>) -> Self {
        Prefab { name, commands }
    }

    pub(crate) fn commands(&self) -> &Vec<PrefabCommand> {
        &self.commands
    }
}

/// A name/value pair representing a field on a type
#[derive(Debug)]
pub(crate) struct ReflectField {
    pub name: String,
    pub value: Box<dyn Reflect>,
}

impl From<ReflectField> for (String, Box<dyn Reflect>) {
    fn from(field: ReflectField) -> Self {
        (field.name, field.value)
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct PrefabComponent {
    name: String,
    dynamic_value: Box<dyn Reflect>,
    #[derivative(Debug = "ignore")]
    reflect: ReflectComponent,
    type_id: TypeId,
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
    pub fn new(
        name: &str,
        root: Box<dyn Reflect>,
        reflect: ReflectComponent,
        type_id: TypeId,
    ) -> Self {
        PrefabComponent {
            name: name.to_string(),
            dynamic_value: root,
            reflect,
            type_id,
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn root(&self) -> &dyn Reflect {
        self.dynamic_value.as_ref()
    }

    #[allow(dead_code)]
    pub fn root_mut(&mut self) -> &mut Box<dyn Reflect> {
        &mut self.dynamic_value
    }

    pub fn reflect(&self) -> &ReflectComponent {
        &self.reflect
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct PrefabProcessorData {
    key: String,
    #[derivative(Debug = "ignore")]
    properties: Option<DynamicStruct>,
    #[derivative(Debug = "ignore")]
    processor: Arc<dyn PrefabProcessor + Send + Sync + 'static>,
}

impl PrefabProcessorData {
    pub fn new(
        key: &str,
        properties: Option<DynamicStruct>,
        processor: Arc<dyn PrefabProcessor + Send + Sync + 'static>,
    ) -> Self {
        Self {
            key: key.to_string(),
            properties,
            processor,
        }
    }

    #[allow(dead_code)]
    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    pub fn properties(&self) -> Option<&DynamicStruct> {
        self.properties.as_ref()
    }

    pub fn processor(&self) -> &Arc<dyn PrefabProcessor + Send + Sync + 'static> {
        &self.processor
    }
}

#[derive(Debug)]
pub(crate) struct PrefabLoad {
    file_name: String,
}

impl PrefabLoad {
    pub fn new(name: &str) -> Self {
        PrefabLoad {
            file_name: name.to_string(),
        }
    }

    pub fn path(&self) -> &str {
        self.file_name.as_str()
    }
}

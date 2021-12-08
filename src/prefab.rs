use bevy::{
    prelude::*,
    reflect::DynamicStruct,
};

use derivative::*;

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct Prefab {
    name: Option<String>,
    components: Vec<PrefabComponent>,
    processors: Option<Vec<PrefabProcessorData>>,
}

impl Prefab {
    pub fn new(
        name: Option<String>,
        components: Vec<PrefabComponent>,
        processors: Option<Vec<PrefabProcessorData>>,
    ) -> Self {
        Prefab {
            name,
            components,
            processors,
        }
    }

    pub fn components(&self) -> &Vec<PrefabComponent> {
        &self.components
    }

    pub fn processors(&self) -> Option<&Vec<PrefabProcessorData>> {
        self.processors.as_ref()
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

#[derive(Debug)]
pub(crate) struct PrefabComponent {
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
    pub fn new(name: &str, root: Box<dyn Reflect>) -> Self {
        PrefabComponent {
            name: name.to_string(),
            dynamic_value: root,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn root(&self) -> &dyn Reflect {
        self.dynamic_value.as_ref()
    }

    pub fn root_mut(&mut self) -> &mut Box<dyn Reflect> {
        &mut self.dynamic_value
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct PrefabProcessorData {
    key: String,
    #[derivative(Debug = "ignore")]
    properties: Option<DynamicStruct>,
}

impl PrefabProcessorData {
    pub fn new(key: &str, properties: Option<DynamicStruct>) -> Self {
        Self {
            key: key.to_string(),
            properties,
        }
    }

    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    pub fn properties(&self) -> Option<&DynamicStruct> {
        self.properties.as_ref()
    }
}

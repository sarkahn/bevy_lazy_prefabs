use bevy::{
    prelude::*,
    reflect::{TypeUuid},
};

#[derive(Debug, TypeUuid)]
#[uuid = "289f0b4a-2b90-49d2-af63-61ad2fec867c"]
pub struct Prefab {
    name: Option<String>,
    components: Vec<PrefabComponent>,
}

impl Prefab {
    pub fn new(name: Option<String>, components: Vec<PrefabComponent>) -> Self {
        Prefab { name, components }
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
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
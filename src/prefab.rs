use bevy::{prelude::*, reflect::{TypeUuid, DynamicStruct}};

use derivative::*;

use crate::{bundle::PrefabBundle, PrefabMaterial};

#[derive(Derivative)]
#[derivative(Debug)]
#[derive(TypeUuid)]
#[uuid = "289f0b4a-2b90-49d2-af63-61ad2fec867c"]
pub struct Prefab {
    name: Option<String>,
    components: Vec<PrefabComponent>,
    processors: Option<Vec<PrefabProcessorData>>,
    //#[derivative(Debug="ignore")]
    //bundles: Option<Vec<PrefabBundle>>,
    //material: Option<PrefabMaterial>,
}

impl Prefab {
    pub fn new(
        name: Option<String>,
        components: Vec<PrefabComponent>,
        _bundles: Option<Vec<PrefabBundle>>,
        _assets: Option<PrefabMaterial>,
        processors: Option<Vec<PrefabProcessorData>>,
    ) -> Self {
        Prefab {
            name,
            components, /*bundles, material: assets*/
            processors,
        }
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

    pub fn processors(&self) -> Option<&Vec<PrefabProcessorData>> {
        self.processors.as_ref()
    }

    // pub fn material(&self) -> Option<&PrefabMaterial> {
    //     self.material.as_ref()
    // }

    // pub fn bundles(&self) -> Option<&Vec<PrefabBundle>> {
    //     self.bundles.as_ref()
    // }

    // pub fn take_material(&mut self) -> PrefabMaterial {
    //     self.material.take().unwrap()
    // }
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


#[derive(Derivative)]
#[derivative(Debug)]
pub struct PrefabProcessorData {
    key: String,
    #[derivative(Debug="ignore")]
    properties: Option<DynamicStruct>,
}

impl PrefabProcessorData {
    pub fn new(key: &str, properties: Option<DynamicStruct>) -> Self 
    {
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
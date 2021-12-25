use std::sync::Arc;

use bevy::{
    prelude::*,
    reflect::{DynamicStruct, TypeUuid},
};
use derivative::*;

#[derive(Derivative, TypeUuid)]
#[derivative(Debug)]
#[uuid = "6ea14da5-6bf8-3ea1-9886-1d7bf6c17d2f"]
pub struct Prefab {
    pub steps: Vec<PrefabBuildStep>,
}

#[derive(Debug, Clone)]
pub enum PrefabBuildStep {
    AddComponent(Arc<PrefabComponent>),
    RunCommand(Arc<PrefabCommandData>),
}

#[derive(Debug)]
pub struct PrefabComponent {
    pub type_name: String,
    pub reflect: Box<dyn Reflect>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct PrefabCommandData {
    pub command_name: String,
    #[derivative(Debug = "ignore")]
    pub properties: Option<DynamicStruct>,
}

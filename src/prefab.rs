use std::sync::Arc;

use bevy::{
    prelude::*,
    reflect::{DynamicStruct, TypeUuid},
};
use derivative::*;

/// An asset built from a *.prefab* file.
///
/// Prefabs can be retrieved from the [crate::PrefabRegistry] and applied to entities
/// via [Commands].
///
/// ## Example
///
/// ```
/// use bevy::prelude::*;
/// use bevy_lazy_prefabs::*;
///
/// fn setup(mut commands: Commands, mut registry: ResMut<PrefabRegistry>) {
///     let prefab = registry.load("some_prefab.prefab").unwrap();
///     commands.spawn().insert_prefab(prefab);
/// }
/// ```
#[derive(Debug, TypeUuid)]
#[uuid = "6ea14da5-6bf8-3ea1-9886-1d7bf6c17d2f"]
pub struct Prefab {
    #[allow(dead_code)]
    pub(crate) name: Option<String>,
    pub(crate) steps: Vec<PrefabBuildStep>,
}

#[derive(Debug)]
pub(crate) enum PrefabBuildStep {
    AddComponent(Arc<PrefabComponent>),
    RunCommand(Arc<PrefabCommandData>),
}

#[derive(Debug)]
pub(crate) struct PrefabComponent {
    pub type_name: String,
    pub reflect: Box<dyn Reflect>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct PrefabCommandData {
    pub name: String,
    #[derivative(Debug = "ignore")]
    pub properties: Option<DynamicStruct>,
}

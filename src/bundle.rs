use bevy::{ecs::world::EntityMut, prelude::*, reflect::DynamicStruct};

use crate::prefab::PrefabComponent;

pub const SPRITE_BUNDLE_LOADER_KEY: &str = "SpriteBundle";
pub const ORTHOGRAPHIC_CAMERA_LOADER_KEY: &str = "OrthographicCameraBundle";

pub struct PrefabBundle {
    name: String,
    components: Option<Vec<PrefabComponent>>,
}

impl PrefabBundle {
    pub fn new(name: &str, components: Option<Vec<PrefabComponent>>) -> Self {
        PrefabBundle {
            name: name.to_string(),
            components,
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn components(&self) -> Option<&Vec<PrefabComponent>> {
        self.components.as_ref()
    }
}

/// Describes how a bundle gets inserted onto an entity.
pub trait PrefabBundleLoader {
    fn key(&self) -> &str;
    fn add_bundle(&self, properties: &DynamicStruct, entity: &mut EntityMut);
}

#[derive(Default)]
pub struct SpriteBundleLoader;
impl PrefabBundleLoader for SpriteBundleLoader {
    fn add_bundle(&self, _properties: &DynamicStruct, entity: &mut EntityMut) {
        entity.insert_bundle(SpriteBundle::default());
    }

    fn key(&self) -> &str {
        SPRITE_BUNDLE_LOADER_KEY
    }
}

#[derive(Default)]
pub struct OrthographicCameraLoader;
impl PrefabBundleLoader for OrthographicCameraLoader {
    fn key(&self) -> &str {
        ORTHOGRAPHIC_CAMERA_LOADER_KEY
    }

    fn add_bundle(&self, _properties: &DynamicStruct, entity: &mut EntityMut) {
        entity.insert_bundle(OrthographicCameraBundle::new_2d());
    }
}

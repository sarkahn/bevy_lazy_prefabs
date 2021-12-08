use bevy::{prelude::*, ecs::world::EntityMut, utils::HashMap};

pub const SPRITE_BUNDLE_LOADER_KEY: &str = "SpriteBundle";
pub const ORTHOGRAPHIC_CAMERA_LOADER_KEY: &str = "OrthographicCameraBundle";

pub struct PrefabBundle {
    name: String,
}

impl PrefabBundle {
    pub fn new(name: &str) -> Self {
        PrefabBundle {
            name: name.to_string()
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

pub trait PrefabBundleLoader {
    fn key(&self) -> &str;
    fn add_bundle(&self, entity: &mut EntityMut);
}

pub struct SpriteBundleLoader;
impl PrefabBundleLoader for SpriteBundleLoader {
    fn add_bundle(&self, entity: &mut EntityMut) {
        entity.insert_bundle(SpriteBundle::default());
    }

    fn key(&self) -> &str {
        SPRITE_BUNDLE_LOADER_KEY
    }
}

pub struct OrthographicCameraLoader;
impl PrefabBundleLoader for OrthographicCameraLoader {
    fn key(&self) -> &str {
        ORTHOGRAPHIC_CAMERA_LOADER_KEY
    }

    fn add_bundle(&self, entity: &mut EntityMut) {
        entity.insert_bundle(OrthographicCameraBundle::new_2d());
    }
}

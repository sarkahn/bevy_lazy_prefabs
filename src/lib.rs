mod commands;
mod dynamic_cast;
mod parse;
mod prefab;
mod registry;

use bevy::prelude::*;

pub use commands::SpawnPrefabCommands;

pub use registry::{PrefabRegistry as PrefabRegistryInternal, PrefabRegistryArc as PrefabRegistry};

pub struct LazyPrefabsPlugin;
impl Plugin for LazyPrefabsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(plugins::LazyPrefabsMinimalPlugin)
        .add_plugin(plugins::LazyPrefabsBevyTypesPlugin)
        ;
    }
}

pub mod plugins {
    use bevy::{prelude::*, sprite::SpritePlugin, render::render_graph::base::MainPass};

    use crate::registry;

    pub struct LazyPrefabsMinimalPlugin;
    impl Plugin for LazyPrefabsMinimalPlugin {
        fn build(&self, app: &mut AppBuilder) {
            app.init_resource::<registry::PrefabRegistryArc>();
        }
    }
    
    pub struct LazyPrefabsBevyTypesPlugin;
    impl Plugin for LazyPrefabsBevyTypesPlugin {
        fn build(&self, app: &mut AppBuilder) {
            let reg = app.world_mut().get_resource::<registry::PrefabRegistryArc>().unwrap();
            let mut reg = reg.write();
    
            reg.register_type::<Transform>();
            reg.register_type::<GlobalTransform>();
            reg.register_type::<Handle<Mesh>>();
            reg.register_type::<Color>();
            reg.register_type::<Vec3>();
            reg.register_type::<Vec2>();
            reg.register_type::<Visible>();
            reg.register_type::<Handle<Mesh>>();
            reg.register_type::<RenderPipelines>();
            reg.register_type::<Draw>();
            reg.register_type::<MainPass>();
        }
    }

    pub struct LazyPrefabsBevy2DPlugin;
    impl Plugin for LazyPrefabsBevy2DPlugin {
        fn build(&self, app: &mut AppBuilder) {
            let reg = app.world_mut().get_resource::<registry::PrefabRegistryArc>().unwrap();
            let mut reg = reg.write();

            reg.register_type::<Sprite>();
            reg.register_type::<Handle<ColorMaterial>>();

            // TODO Special handling for unregisterable types/types that need initialization (like meshes, etc)
            //reg.register_type::<TextureAtlasSprite>();
            reg.register_type::<Handle<TextureAtlas>>();
        }
    }
}


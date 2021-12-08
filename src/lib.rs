mod commands;
mod dynamic_cast;
mod parse;
mod prefab;
mod registry;
mod process;
mod material;
mod bundle;

use bevy::prelude::*;

pub use commands::SpawnPrefabCommands;
pub use material::PrefabMaterial;
pub use material::COLOR_MATERIAL_LOADER_KEY;
pub use material::AddMaterialLoader;

pub use registry::PrefabRegistry;

pub struct LazyPrefabsPlugin {

}

impl Plugin for LazyPrefabsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(plugins::LazyPrefabsMinimalPlugin)
        .add_plugin(plugins::LazyPrefabsBevyTypesPlugin)
        ;
    }
}

pub mod plugins {
    use bevy::{prelude::*, render::render_graph::base::MainPass};

    use crate::{
        PrefabRegistry,
        //PrefabMaterialRegistry,
        material::{ PrefabColorMaterialLoader, AddMaterialLoader}};

    pub struct LazyPrefabsMinimalPlugin;
    impl Plugin for LazyPrefabsMinimalPlugin {
        fn build(&self, app: &mut AppBuilder) {
            app.init_resource::<PrefabRegistry>()
            //.add_prefab_material_loader::<PrefabColorMaterialLoader, ColorMaterial>()
            //.init_resource::<PrefabMaterialRegistry>()
            //.add_system(prefab_load_material_system::<ColorMaterial>.system())
            //.add_system(prefab_load_material_system::<StandardMaterial>.system())
            ;
        }
    }
    
    pub struct LazyPrefabsBevyTypesPlugin;
    impl Plugin for LazyPrefabsBevyTypesPlugin {
        fn build(&self, app: &mut AppBuilder) {
            //let reg = app.world_mut().get_resource::<registry::PrefabRegistryArc>().unwrap();
            //let mut reg = reg.write();
            let mut reg = app.world_mut().get_resource_mut::<PrefabRegistry>().unwrap();
    
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
            let mut reg = app.world_mut().get_resource_mut::<PrefabRegistry>().unwrap();

            reg.register_type::<Sprite>();
            reg.register_type::<Handle<ColorMaterial>>();

            // TODO Special handling for unregisterable types/types that need initialization (like meshes, etc)
            //reg.register_type::<TextureAtlasSprite>();
            reg.register_type::<Handle<TextureAtlas>>();

            app.add_prefab_material_loader::<PrefabColorMaterialLoader, ColorMaterial>();
        }
    }
}

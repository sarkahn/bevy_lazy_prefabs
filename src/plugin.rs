use bevy::{
    prelude::*,
    render::{
        camera::{Camera, OrthographicProjection},
        render_graph::base::MainPass,
    },
};

use crate::{
    build_commands::*,
    PrefabRegistry,
};

/// Default plugin, registers many built-in bevy types and bundles and includes
/// prefab commands for common assets.
pub struct LazyPrefabsPlugin;

impl Plugin for LazyPrefabsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(LazyPrefabsMinimalPlugin)
            .add_plugin(LazyPrefabsCommonTypesPlugin)
            .add_plugin(LazyPrefabsBevy3DPlugin)
            .add_plugin(LazyPrefabsBevy2DPlugin);
    }
}

pub struct LazyPrefabsMinimalPlugin;
impl Plugin for LazyPrefabsMinimalPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PrefabRegistry>();
    }
}

pub struct LazyPrefabsCommonTypesPlugin;
impl Plugin for LazyPrefabsCommonTypesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let mut reg = app
            .world_mut()
            .get_resource_mut::<PrefabRegistry>()
            .unwrap();

        reg.register_type::<Transform>();
        reg.register_type::<GlobalTransform>();
        reg.register_type::<Color>();
        reg.register_type::<Vec3>();
        reg.register_type::<Vec2>();
        reg.register_type::<Camera>();

        reg.register_build_command::<LoadPrefab>();
    }
}

pub struct LazyPrefabsBevy3DPlugin;
impl Plugin for LazyPrefabsBevy3DPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let mut reg = app
            .world_mut()
            .get_resource_mut::<PrefabRegistry>()
            .unwrap();
        reg.register_type::<Visible>();
        reg.register_type::<Handle<Mesh>>();
        reg.register_type::<RenderPipelines>();
        reg.register_type::<Draw>();
        reg.register_type::<MainPass>();

        reg.register_build_command::<InsertPbrBundle>();
        reg.register_build_command::<InsertPerspectiveCameraBundle>();
    }
}

pub struct LazyPrefabsBevy2DPlugin;
impl Plugin for LazyPrefabsBevy2DPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let mut reg = app
            .world_mut()
            .get_resource_mut::<PrefabRegistry>()
            .unwrap();

        reg.register_type::<Sprite>();
        reg.register_type::<OrthographicProjection>();
        reg.register_type::<Handle<ColorMaterial>>();
        reg.register_type::<Handle<TextureAtlas>>();

        reg.register_build_command::<SetColorMaterial>();
        reg.register_build_command::<InsertSpriteBundle>();
        reg.register_build_command::<InsertOrthographicCameraBundle>();
    }
}

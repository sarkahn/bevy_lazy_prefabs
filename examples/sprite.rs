use bevy::prelude::*;
use bevy_lazy_prefabs::{
    plugins::{LazyPrefabsBevy2DPlugin, LazyPrefabsMinimalPlugin},
    SpawnPrefabCommands,
};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(LazyPrefabsMinimalPlugin)
        .add_plugin(LazyPrefabsBevy2DPlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_prefab("sprite.prefab");
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

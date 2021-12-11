use bevy::{prelude::*, reflect::{GetTypeRegistration, TypeRegistry, DynamicStruct}};
use bevy_lazy_prefabs::*;

fn setup(
    mut commands: Commands,
) {
    commands.spawn_prefab("blue_bird.prefab");
    commands.spawn_prefab("camera.prefab");
}

fn main() {
    App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(LazyPrefabsPlugin)
    .add_startup_system(setup.system())
    .run();
}



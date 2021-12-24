use bevy::prelude::{*};
use bevy_lazy_prefabs::*;

fn setup(
    mut commands: Commands,
) {
    //commands.spawn_prefab("cube.prefab");
    //commands.spawn_prefab("persp_camera.prefab");
}

fn main() {
    App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(LazyPrefabsPlugin)
    .add_startup_system(setup.system())
    .run();
}
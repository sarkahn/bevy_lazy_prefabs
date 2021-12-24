use bevy::prelude::*;
use bevy_lazy_prefabs::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(LazyPrefabsPlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut registry: ResMut<PrefabRegistry>
) {
    commands.spawn_prefab("sprite.prefab", &mut registry).unwrap();
    commands.spawn_prefab("ortho_camera.prefab", &mut registry).unwrap();
}

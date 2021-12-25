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
    mut registry: ResMut<PrefabRegistry>, 
) {
    let prefab = registry.load("cube.prefab").unwrap();
    commands.spawn().insert_prefab(prefab);

    let cam = registry.load("cam_3d.prefab").unwrap();
    commands.spawn().insert_prefab(cam);
}
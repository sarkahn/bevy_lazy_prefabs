use bevy::prelude::*;
use bevy_lazy_prefabs::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(LazyPrefabsPlugin)
        .add_startup_system(setup.system())
        .add_startup_system_to_stage(StartupStage::PostStartup, check.system())
        .run();
}

fn setup(mut commands: Commands, mut registry: ResMut<PrefabRegistry>) {
    let hello = registry.load("hello_world.prefab").unwrap();
    commands.spawn().insert_prefab(hello);
}

fn check(
    query: Query<&Transform>,
) {
    let t = query.single().unwrap();
    println!("Resulting position: {}", t.translation);
}

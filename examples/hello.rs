use bevy::prelude::*;
use bevy_lazy_prefabs::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(LazyPrefabsPlugin)
        .add_startup_system(setup.system())
        .add_system(check.system())
        .run();
}

fn setup(mut commands: Commands, mut registry: ResMut<PrefabRegistry>) {
    let hello = registry.load("hello_world.prefab").unwrap();
    commands.spawn().insert_prefab(hello);
}

fn check(
    input: Res<Input<KeyCode>>,
    query: Query<&Transform>,
) {
    if input.just_pressed(KeyCode::Space) {
        let t = query.single().unwrap();
        println!("Resulting position: {}", t.translation);
    }
}

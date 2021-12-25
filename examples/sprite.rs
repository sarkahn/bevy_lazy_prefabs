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

fn setup(
    mut commands: Commands,
    mut registry: ResMut<PrefabRegistry>
) {
    let prefab = registry.load("sprite.prefab").unwrap(); 
    commands.spawn().insert_prefab(prefab);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn check(
    input: Res<Input<KeyCode>>,
    q: Query<&Transform>,
) {
    if input.just_pressed(KeyCode::Space) {
        let t = q.single().unwrap();

        println!("Transform: {}", t.translation);
    }
}

use bevy::prelude::*;
use bevy_lazy_prefabs::*;

fn setup(mut commands: Commands) {
    commands.spawn_prefab("handle.prefab");
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn query(
    input: Res<Input<KeyCode>>,
    materials: Res<Assets<ColorMaterial>>,
    q: Query<&Handle<ColorMaterial>>,
) {
    if input.just_pressed(KeyCode::Space) {
        for handle in q.iter() {
            println!("Handle {:#?}", handle);
            let mat = materials.get(handle).unwrap();
            println!("Color {:#?}", mat.color);
        }
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(plugins::LazyPrefabsMinimalPlugin)
        .add_plugin(plugins::LazyPrefabsBevy2DPlugin)
        .add_startup_system(setup.system())
        .add_system(query.system())
        .run();
}

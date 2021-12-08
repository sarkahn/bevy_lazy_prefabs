use bevy::prelude::*;
use bevy_lazy_prefabs::*;


fn setup(
    mut registry: ResMut<PrefabRegistry>,
    mut commands: Commands
) {
    registry.register_type::<Transform>();
    registry.register_type::<Vec3>();

    commands.spawn_prefab("handle.prefab");
    commands.spawn_prefab("camera.prefab");
}

fn query(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    input: Res<Input<KeyCode>>, 
    mut q: Query<&mut Handle<ColorMaterial>>,
) {
    let tex = materials.add(asset_server.load("icon.png").into());
    if input.just_pressed(KeyCode::Space) {
        let mut mat = q.single_mut().unwrap();
        *mat = tex.clone();
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

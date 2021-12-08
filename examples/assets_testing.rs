use bevy::{prelude::*, reflect::DynamicStruct};
use bevy_lazy_prefabs::{plugins::{LazyPrefabsBevy2DPlugin, LazyPrefabsMinimalPlugin}, PrefabMaterial, COLOR_MATERIAL_LOADER_KEY};

fn main() {
    App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(LazyPrefabsMinimalPlugin)
    .add_plugin(LazyPrefabsBevy2DPlugin)
    .add_startup_system(setup.system())
    .run();
}

fn setup(
    mut commands: Commands,
) {
    let mut color = DynamicStruct::default();
    color
    .insert("color", Color::GREEN);
    let mat = PrefabMaterial::new(
        "icon.png",
        COLOR_MATERIAL_LOADER_KEY.into(),
        Some(color)
    );

    commands.spawn()
    .insert_bundle(SpriteBundle::default())
    .insert(mat)
    .insert(Handle::<ColorMaterial>::default());

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}




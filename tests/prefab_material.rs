use bevy_lazy_prefabs::{*, plugins::LazyPrefabsMinimalPlugin};
use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use bevy::sprite::SpritePlugin;

// Can't work - need access to textures at a minimum, textures are in bevy render, bevy render requires a window
// to render to

// #[test]
// fn test() {
//     App::build()
//     .add_plugins(MinimalPlugins)
//     .add_plugin(AssetPlugin::default())
//     .add_plugin(LazyPrefabsMinimalPlugin)
//     .set_runner(runner)
//     .run();
// }

// fn runner(mut app: App) {
//     app.update();
// }

// fn setup(
//     mut commands: Commands,
// ) {
// }


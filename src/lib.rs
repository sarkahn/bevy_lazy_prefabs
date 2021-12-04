mod commands;
mod dynamic_cast;
mod parse;
mod prefab;
mod registry;

use bevy::prelude::*;

pub use commands::SpawnPrefabCommands;

pub use registry::{PrefabRegistry as PrefabRegistryInternal, PrefabRegistryArc as PrefabRegistry};

pub struct LazyPrefabsPlugin;
impl Plugin for LazyPrefabsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<registry::PrefabRegistryArc>();
    }
}

mod bevy_commands;
mod commands;
mod dynamic_cast;
mod parse;
mod prefab;
mod registry;
mod plugin;

pub use registry::PrefabRegistry;
pub use plugin::LazyPrefabsPlugin;
pub use bevy_commands::SpawnPrefabCommands;

pub mod prefab_commands {
    /// Hi
    pub use crate::commands::InsertSpriteBundle;
}

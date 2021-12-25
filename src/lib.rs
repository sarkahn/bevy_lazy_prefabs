//! A crate for simple human readable/writable prefab text files in bevy.
//! 
//! # .Prefab Files
//! 
//! First, write a *.prefab* file and put it in the *assets/prefabs* directory. 
//! ```ignore
//! SomePrefab {                   // Prefab name is optional. Outer braces are required. 
//!     Transform {                // Components are listed by type name.
//!         translation : Vec3 {   // Component fields can be initialized inside nested curly braces.
//!             x: 15.0, y: 10.5,  // Any omitted fields will be initialized to default.
//!         },
//!     },
//!     Visible,                   // If you choose not to initialize any fields, the braces can be omitted entirely.
//!     Draw,
//! }
//! ```
//! 
//! In the above example we are authoring a prefab with Transform, Visible and Draw components.
//! In this case the entity's transform will be initialized to position (15.0,10.0,0.0) when entity is spawned.
//! This isn't much use though - the above entity won't be rendered since it has no mesh or material. 
//! For that we can use a [PrefabProcessor].
//! 
//! # Processors
//! 
//! Prefab processors allow you to include complex components that require extra steps
//! to correctly initialize, such as meshes, materials, or bundles.
//! 
//! Custom processors can be authored, but there are several included for more common components.
//! 
//! ```ignore
//! {
//!     processor!(SpriteBundle {       // Processors are specified by `processor!(key)`
//!         texture_path: "alien.png",  // The SpriteBundle processor will read the `texture_path`
//!         color: Color::RED,          // And `color` properties from the *.prefab* file
//!     })
//! }
//! ```
//! 
//! The above *.prefab* file will result in an entity with a `SpriteBundle`. The sprite bundle's `ColorMaterial`
//! component will be initialized with the given texture and color. Note these 'fields' are not referring directly to fields
//! in the bundle, but are optional properties that get passed to the processor and used in the initialization process.
//! 
//! # Spawning A Prefab
//! 
//! Once you have your *.prefab* file in the *assets/prefabs* directory you can spawn a prefab via `Commands`:
//! 
//! ```
//! use bevy::prelude::*;
//! use bevy_lazy_prefabs::*;
//! 
//! fn setup(mut commands: Commands, mut registry: ResMut<PrefabRegistry>) {
//!   let sprite = registry.load("sprite.prefab").unwrap();
//!   commands.spawn().insert_prefab(sprite);
//!   let cam = registry.load("cam_2d.prefab").unwrap();
//!   commands.spawn().insert_prefab(cam);
//!  }
//! ``` 

mod bevy_commands;
mod build_commands;
mod dynamic_cast;
mod parse;
mod plugin;
mod prefab;
mod registry;

pub use bevy_commands::SpawnPrefabCommands;
pub use plugin::LazyPrefabsPlugin;
pub use registry::PrefabRegistry;

pub mod prefab_commands {
    /// Hi
    pub use crate::build_commands::InsertSpriteBundle;
}

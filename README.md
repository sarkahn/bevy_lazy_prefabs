[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/bevy_lazy_prefabs)](https://crates.io/crates/bevy_lazy_prefabs)
[![docs](https://docs.rs/bevy_lazy_prefabs/badge.svg)](https://docs.rs/bevy_lazy_prefabs/)

# Bevy Lazy Prefabs

A crate for simple human readable/writable prefab text files in bevy.

Note: This is not intended to be a flexible and long-term prefab solution, but should serve well for
simple games and prototyping to avoid having to define your entities entirely in code.

# .Prefab Files

First, write a *.prefab* file and put it in the *assets/prefabs* directory. 
```rust
SomePrefab {                   // Prefab name is optional. Outer braces are required. 
    Transform {                // Components are listed by type name.
        translation : Vec3 {   // Component fields can be initialized inside nested curly braces.
            x: 15.0, y: 10.5,  // Any omitted fields will be initialized to default.
        },
    },
    Visible,                   // If you choose not to initialize any fields, the braces can be omitted entirely.
    Draw,
    SomeComponent {            // Custom components are supported
        some_int: 15,
    },
}
```

In the above example we are authoring a prefab with `Transform`, `Visible`, `Draw`, and `SomeComponent` components.
In this case the entity's transform will be initialized to position (15.0,10.5,0.0) when the entity is spawned.

Custom components will only work in prefabs if they derive `Reflect` and `Default`, and if they have the 
`#[reflect(Component)]` attribute. Most built in bevy types already meet this constraint. They must also be 
registered with the `PrefabRegistry` during setup.

The above prefab isn't much use though - the entity won't be rendered since it has no mesh or material. 
For that we can use a `BuildPrefabCommand`.

# BuildPrefabCommands

Build commands allow you to include complex components that require extra steps to correctly initialize, 
such as meshes, materials, or bundles.

Custom commands can be authored, but there are several included for more common components:
- `InsertSpriteBundle` - Inserts a `SpriteBundle` on an entity. Can specify `color` and `texture_path`.
- `SetColorMaterial` - Modify an existing `ColorMaterial` on the entity.
- `LoadPrefab` - Load an existing prefab and perform it's build steps on the current entity. 
- `InsertPbrBundle` - Inserts a `PbrBundle`. Can specify mesh `shape`, `size`, and `flip`.
- `InsertOrthographicCameraBundle` - Inserts an `OrthographicCameraBundle`. Can specify `scale`.
- `InsertPerspectiveCameraBundle` - Inserts a `PerspectiveCameraBundle`. Can specify `position` and `looking_at`.


## Example

```rust
{
    InsertSpriteBundle! (          
        texture_path: "alien.png", 
        color: Color::RED,         
    ),
}
```

The above *.prefab* file will result in an entity with all the components from a  `SpriteBundle`. The sprite bundle's 
`ColorMaterial` component will be initialized with the given texture and color. 

Note these 'fields' are not referring directly to fields in the bundle, but are optional properties that get passed 
to the build command and used in the initialization process. How these properties get used is defined by every 
individual build command.

# Spawning A Prefab

Once you have your *.prefab* file in the *assets/prefabs* directory you can spawn a prefab using the 
`PrefabRegistry` and `Commands`:

```rust
use bevy::prelude::*;
use bevy_lazy_prefabs::*;

fn setup(mut commands: Commands, mut registry: ResMut<PrefabRegistry>) {
  let sprite = registry.load("sprite.prefab").unwrap();
  commands.spawn().insert_prefab(sprite);
  let cam = registry.load("cam_2d.prefab").unwrap();
  commands.spawn().insert_prefab(cam);
 }
``` 

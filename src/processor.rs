use bevy::{ecs::world::EntityMut, prelude::*, reflect::DynamicStruct};

use crate::dynamic_cast::GetValue;

pub const COLOR_MATERIAL_PROCESSOR_KEY: &str = "ColorMaterial";
pub const SPRITE_BUNDLE_PROCESSOR_KEY: &str = "SpriteBundle";

/// A processor for handling more complex prefab data.
/// 
/// A prefab processor can perform complex initialization on prefabs that can't
/// reasonably be handled from a text file. This includes things like inserting bundles, 
/// loading handles for meshes and materials, and generally any other kind of asset or
/// property that requires external data.
///
/// Note that processors may rely on ecs systems, in which case  their effects won't happen
/// until after the prefab is spawned.
pub trait PrefabProcessor {
    /// The key for this processor. This is the name you refer to the processor by
    /// from your `.prefab` file.
    fn key(&self) -> &str;

    /// Process the prefab.
    ///
    /// ### Arguments
    ///
    ///  * `properties` - An optional  [DynamicStruct] containing any properties read 
    /// from the `.prefab` file. [None] if no properties were receieved.
    ///  * `entity` - The prefab entity, to be modified as needed.
    fn process_prefab(&self, properties: Option<&DynamicStruct>, entity: &mut EntityMut);
}

pub(crate) struct AddColorMaterial {
    color: Option<Color>,
    texture_path: Option<String>,
}

/// A processor for a Handle<ColorMaterial>.
///
/// ## Optional Properties:
///
/// * `color` - The color for the material.
/// * `texture_path` - The path to the texture for the material.
#[derive(Default)]
pub(crate) struct ColorMaterialProcessor;

impl PrefabProcessor for ColorMaterialProcessor {
    fn process_prefab(&self, properties: Option<&DynamicStruct>, entity: &mut EntityMut) {
        let err = "Error loading ColorMaterial, missing required property 'texture_path'";

        if let Some(properties) = properties {
            let tex_path = properties
                .try_get::<String>("texture_path")
                .unwrap_or_else(|_| panic!("{}", err));

            let col = properties.try_get::<Color>("color").ok().cloned();

            entity.insert(AddColorMaterial {
                color: col,
                texture_path: Some(tex_path.to_owned()),
            });
        } else {
            panic!("{}", err);
        }
    }

    fn key(&self) -> &str {
        COLOR_MATERIAL_PROCESSOR_KEY
    }
}

/// Processes the [AddColorMaterial] component to add a ColorMaterial to an entity.
pub(crate) fn load_color_material(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut assets: ResMut<Assets<ColorMaterial>>,
    mut q: Query<(Entity, &mut Handle<ColorMaterial>, &AddColorMaterial)>,
) {
    for (e, mut handle, add) in q.iter_mut() {
        let tex = add.texture_path.as_ref().map(|tex| server.load(tex.as_str()));

        let color = add.color.unwrap_or_default();

        let mat = ColorMaterial {
            texture: tex,
            color,
        };

        *handle = assets.add(mat);

        commands.entity(e).remove::<AddColorMaterial>();
    }
}

/// Processor for sprite bundles.
///
/// ## Optional Properties:
///
/// * `color` - The color for the sprite material.
/// * `texture_path` - The path to the texture for the sprite material.
#[derive(Default)]
pub struct SpriteBundleProcessor;
impl PrefabProcessor for SpriteBundleProcessor {
    fn key(&self) -> &str {
        SPRITE_BUNDLE_PROCESSOR_KEY
    }

    fn process_prefab(&self, properties: Option<&DynamicStruct>, entity: &mut EntityMut) {
        entity.insert_bundle(SpriteBundle::default());

        if let Some(properties) = properties {
            let tex_path = properties.try_get::<String>("texture_path").ok();

            let col = match properties.try_get::<Color>("color") {
                Ok(col) => Some(col.to_owned()),
                Err(_) => None,
            };

            entity.insert(AddColorMaterial {
                color: col,
                texture_path: tex_path.cloned(),
            });
        }
    }
}

#[macro_export]
/// Implement a processor which does nothing but insert a bundle.
macro_rules! impl_bundle_processor {
    ($name:ident, $key:ident, $bundle:expr) => {
        #[derive(Default)]
        pub struct $name;
        impl PrefabProcessor for $name {
            fn key(&self) -> &str {
                stringify!($key)
            }

            fn process_prefab(&self, properties: Option<&DynamicStruct>, entity: &mut EntityMut) {
                entity.insert_bundle($bundle);
                drop(&properties);
            }
        }
    };
}

impl_bundle_processor!(
    OrthographicCameraBundleProcessor,
    OrthographicCameraBundle,
    OrthographicCameraBundle::new_2d()
);

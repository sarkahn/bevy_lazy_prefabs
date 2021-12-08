use std::ops::RangeBounds;

use bevy::{ecs::world::EntityMut, prelude::*, reflect::DynamicStruct};

use crate::dynamic_cast::GetValue;

/// A prefab processor can perform complex initialization on prefabs that can't
/// reasonably be handled from a text file. This includes things like bundles, meshes, materials,
/// and generally any other kind of asset or property that requires external data.
/// 
/// Note that processors may rely on ecs systems, meaning their effects won't happen
/// until after the prefab is spawned.
pub trait PrefabProcessor {
    /// The key for this processor. This is the name you refer to the processor by
    /// from your `.prefab` file.
    fn key(&self) -> &str;

    /// Process the prefab.
    ///
    /// ### Arguments
    ///
    ///  * `properties` - An optional  `DynamicStruct` containing any properties from the
    ///  `.prefab` file.
    ///  * `entity` - The prefab entity, to be modified as needed.
    fn process_prefab(&self, properties: Option<&DynamicStruct>, entity: &mut EntityMut);

    /// Optional method to modify the app on initialization. This could be used for example
    /// to add a system to the app that's specifically tied to this processor.
    fn on_init(&self, app: &mut AppBuilder);
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
pub struct ColorMaterialProcessor;

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

    fn on_init(&self, app: &mut AppBuilder) {
        drop(app);
        //app.add_system(load_color_material.system());
    }

    fn key(&self) -> &str {
        "ColorMaterial"
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
        let tex = match &add.texture_path {
            Some(tex) => Some(server.load(tex.as_str())),
            None => None,
        };

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
/// * `color` - The color for the material.
/// * `texture_path` - The path to the texture for the material.
#[derive(Default)]
pub struct SpriteBundleProcessor;
impl PrefabProcessor for SpriteBundleProcessor {
    fn key(&self) -> &str {
        "SpriteBundle"
    }

    fn process_prefab(&self, properties: Option<&DynamicStruct>, entity: &mut EntityMut) {
        println!("Sprite bundle processor is processing entity");
        entity.insert_bundle(SpriteBundle::default());

        if let Some(properties) = properties {
            let tex_path = properties
                .try_get::<String>("texture_path").ok();

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

    fn on_init(&self, app: &mut AppBuilder) {
        //app.systems.contains(item)
        drop(app)
    }
}

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

            fn on_init(&self, app: &mut AppBuilder) {
                drop(&app)
            }
        }
    };
}

impl_bundle_processor!(
    OrthographicCameraBundleProcessor,
    OrthographicCameraBundle,
    OrthographicCameraBundle::new_2d()
);

//impl_bundle_processor!(SpriteBundleProcessor, SpriteBundle, SpriteBundle::default());

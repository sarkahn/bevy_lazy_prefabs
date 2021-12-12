use bevy::{prelude::*, reflect::DynamicStruct};

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
    fn process_prefab(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity);
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
    fn process_prefab(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        let mut col = None;
        let mut path = None;

        if let Some(properties) = properties {
            if let Ok(tex_path) = properties.try_get::<String>("texture_path") {
                path = Some(tex_path.clone());
            }

            if let Ok(color) = properties.try_get::<Color>("color") {
                col = Some(color.clone());
            }

            if let Some(existing_mat) =  world.get_mut::<Handle<ColorMaterial>>(entity) {
                if col.is_none() && path.is_none() {
                    return;
                }

                let existing_mat = existing_mat.clone_weak();
                world.resource_scope(|world, mut materials: Mut<Assets<ColorMaterial>>| {
                    let mat = materials.get_mut(existing_mat).unwrap();

                    update_mat(world, mat, col, path);
                });
            } else {
                world.resource_scope(|world, mut materials: Mut<Assets<ColorMaterial>>| {
                    let mut mat = ColorMaterial::default();

                    update_mat(world, &mut mat, col, path);

                    let handle = materials.add(mat);

                    world.entity_mut(entity).insert(handle);
                });
            }
        } else {
            world.resource_scope(|world, mut materials: Mut<Assets<ColorMaterial>>| {
                let handle = materials.add(ColorMaterial::default());
                world.entity_mut(entity).insert(handle);
            });
        }
    }

    fn key(&self) -> &str {
        COLOR_MATERIAL_PROCESSOR_KEY
    }
}

fn update_mat(world: &mut World, mat: &mut ColorMaterial, col: Option<Color>, path: Option<String>) {
    if let Some(col) = col {
        mat.color = col;
    }
    if let Some(path) = path {
        let server = world.get_resource::<AssetServer>().unwrap();
        let tex: Handle<Texture> = server.load(path.as_str());
        mat.texture = Some(tex);
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

    fn process_prefab(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        let mat = get_color_material(properties, world);

        let mut entity = world.entity_mut(entity);
        entity.insert_bundle(SpriteBundle::default());

        if let Some(mat) = mat {
            entity.insert(mat);
        }
    }
}

fn get_color_material(
    properties: Option<&DynamicStruct>, 
    world: &mut World
) -> Option<Handle<ColorMaterial>> {
    if let Some(properties) = properties {

        let color = properties.try_get::<Color>("color").ok().cloned();

        if let Ok(tex_path) = properties.try_get::<String>("texture_path") {
            let server = world.get_resource::<AssetServer>().unwrap();
            let texture: Handle<Texture> = server.load(tex_path.as_str()).clone();

            let mat = ColorMaterial {
                texture: Some(texture),
                color: color.unwrap_or_default()
            };

            let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
            return Some(materials.add(mat));
        }
    }
    None
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

            fn process_prefab( &self, 
                properties: Option<&DynamicStruct>, 
                world: &mut World, 
                entity: Entity) 
            {
                let mut entity = world.entity_mut(entity);
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

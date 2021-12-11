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
    fn process_prefab(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity);
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
    fn process_prefab(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        let mut col = None;
        let mut path = None;
        let mut entity = world.entity_mut(entity);

        if let Some(properties) = properties {
            if let Ok(tex_path) = properties.try_get::<String>("texture_path") {
                path = Some(tex_path.clone());
            }

            if let Ok(color) = properties.try_get::<Color>("color") {
                col = Some(color.clone());
            }

            entity.insert(AddColorMaterial {
                color: col,
                texture_path: path,
            });
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
    for (e, mut handle, add_mat) in q.iter_mut() {

        println!("Processing color material system");

        let tex = match &add_mat.texture_path {
            Some(path) => Some(server.load(path.as_str())),
            None => None,
        };
        let color = add_mat.color;

        println!("{:#?}", color);

        if let Some(existing) = assets.get_mut(handle.clone_weak()) {
            if tex.is_some() {
                existing.texture = tex;
            }
            if color.is_some() {
                existing.color = color.unwrap();
            }
        } else {
            *handle = assets.add(ColorMaterial {
                texture: tex,
                color: color.unwrap_or_default()
            });
        }


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

    fn process_prefab(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {

        println!("Inserting sprite bundle");

        let mat = get_color_material(properties, world);

        let mut entity = world.entity_mut(entity);
        entity.insert_bundle(SpriteBundle::default());

        if let Some(mat) = mat {
            entity.insert(mat);
        }

        // if let Some(properties) = properties {
        //     let tex_path = properties.try_get::<String>("texture_path").ok();

        //     let col = match properties.try_get::<Color>("color") {
        //         Ok(col) => Some(col.to_owned()),
        //         Err(_) => None,
        //     };

        //     entity.insert(AddColorMaterial {
        //         color: col,
        //         texture_path: tex_path.cloned(),
        //     });
        // }
    }
}

fn get_color_material(
    properties: Option<&DynamicStruct>, 
    world: &mut World
) -> Option<Handle<ColorMaterial>> {
    if let Some(properties) = properties {

        let color = properties.try_get::<Color>("color").ok();
        let color = color.cloned();

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

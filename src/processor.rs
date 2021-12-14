use bevy::{prelude::*, reflect::DynamicStruct};

use crate::dynamic_cast::GetValue;

pub const COLOR_MATERIAL_PROCESSOR_KEY: &str = "ColorMaterial";
pub const SPRITE_BUNDLE_PROCESSOR_KEY: &str = "SpriteBundle";
pub const ORTHOGRAPHIC_BUNDLE_PROCESSOR_KEY: &str = "OrthographicCameraBundle";
pub const MESH_BUNDLE_PROCESSOR_KEY: &str = "MeshBundle";
pub const PBR_BUNDLE_PROCESSOR_KEY: &str = "PbrBundle";
pub const PERSPECTIVE_CAMERA_BUNDLE_PROCESSOR_KEY: &str = "PerspectiveCameraBundle";

/// A processor for handling more complex prefab entity initialization.
/// 
/// A prefab processor can perform complex initialization on prefab entities that can't
/// reasonably be handled from a text file. This includes things like inserting bundles,
/// loading handles for meshes and materials, and initializing any other kind of asset or
/// property that requires external data.
pub trait PrefabProcessor {
    /// The key for this processor. This is the name you refer to the processor by
    /// from your *.prefab* file.
    fn key(&self) -> &str;

    /// Process and modify the prefab entity as needed.
    ///
    /// ### Arguments
    ///
    ///  - `properties` - An optional  [DynamicStruct] containing any properties read
    /// from the *.prefab* file. [None] if no properties were receieved.
    ///  - `entity` - The prefab entity, to be modified as needed.
    /// 
    /// ### Example 
    /// 
    /// ```ignore
    /// struct Name {
    ///     value: String,
    /// }
    /// 
    /// // Insert a name component whose value is receieved from a *.prefab* file.
    /// pub fn process_prefab(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
    ///     if let Some(props) = properties {
    ///         if let Ok(name) = props.try_get::<String>("name") {
    ///             world.entity_mut(entity).insert(Name { value: name.clone() });
    ///         }
    ///     }
    /// }
    /// ```
    fn process_prefab(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity);
}

/// A processor for a [Handle<ColorMaterial>].
///
/// ### Optional Properties:
///
/// - `color` - The color for the material.
/// - `texture_path` - The path to the texture for the material.
#[derive(Default)]
pub(crate) struct ColorMaterialProcessor;

impl PrefabProcessor for ColorMaterialProcessor {
    fn process_prefab(
        &self,
        properties: Option<&DynamicStruct>,
        world: &mut World,
        entity: Entity,
    ) {
        let (color,path) = get_material_props(properties);
 
        if let Some(existing_mat) = world.get_mut::<Handle<ColorMaterial>>(entity) {
            let existing_mat = existing_mat.clone_weak();
            world.resource_scope(|world, mut materials: Mut<Assets<ColorMaterial>>| {
                let mat = materials.get_mut(existing_mat).unwrap();

                if let Some(col) = color {
                    mat.color = col.clone();
                }
                if let Some(path) = path {
                    let server = world.get_resource::<AssetServer>().unwrap();
                    let tex: Handle<Texture> = server.load(path.as_str());
                    mat.texture = Some(tex);
                }
            });
        } else {
            let handle = get_material(world, (color,path)).unwrap_or_default();

            world.entity_mut(entity).insert(handle);
        }
    }

    fn key(&self) -> &str {
        COLOR_MATERIAL_PROCESSOR_KEY
    }
}

/// Processor for sprite bundles.
///
/// ### Optional Properties:
///
/// - `color` - The color for the sprite material.
/// - `texture_path` - The path to the texture for the sprite material.
#[derive(Default)]
pub struct SpriteBundleProcessor;
impl PrefabProcessor for SpriteBundleProcessor {
    fn key(&self) -> &str {
        SPRITE_BUNDLE_PROCESSOR_KEY
    }

    fn process_prefab(
        &self,
        properties: Option<&DynamicStruct>,
        world: &mut World,
        entity: Entity,
    ) {
        let (color,path) = get_material_props(properties);
        let mat = get_material(world, (color,path)); 

        let mut entity = world.entity_mut(entity);
        entity.insert_bundle(SpriteBundle::default());

        if let Some(mat) = mat {
            entity.insert(mat);
        }
    }
}

fn get_material_props(
    properties: Option<&DynamicStruct>,
) -> (Option<&Color>, Option<&String>) {
    if let Some(properties) = properties {
        let color = properties.try_get::<Color>("color").ok();
        let tex_path = properties.try_get::<String>("texture_path").ok();

        return (color, tex_path);
    } 
    (None,None)
}

fn get_material(
    world: &mut World,
    material_props: (Option<&Color>, Option<&String>),
) -> Option<Handle<ColorMaterial>> {
    let (col,path) = material_props;

    let tex: Option<Handle<Texture>> = match path {
        Some(path) => {
            let server = world.get_resource::<AssetServer>().unwrap();
            Some(server.load(path.as_str()).clone())
        }
        None => None
    };

    if col.is_none() && tex.is_none() {
        return None;
    }

    let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
    let mat = ColorMaterial {
        texture: tex,
        color: col.cloned().unwrap_or_default(),
    };
    return Some(materials.add(mat));
}

#[derive(Default)]
pub struct MeshBundleProcessor;
impl PrefabProcessor for MeshBundleProcessor {
    fn key(&self) -> &str {
        MESH_BUNDLE_PROCESSOR_KEY
    }

    fn process_prefab(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        world.entity_mut(entity).insert_bundle(MeshBundle::default());
        
        if let Some(mesh) = get_mesh(properties) {
            world.resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                let handle = meshes.add(mesh);
                world.entity_mut(entity).insert(handle);
            });
        }
    }
}

#[derive(Default)]
pub struct PbrBundleProcessor;
impl PrefabProcessor for PbrBundleProcessor {
    fn key(&self) -> &str {
        PBR_BUNDLE_PROCESSOR_KEY
    }

    fn process_prefab(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        world.entity_mut(entity).insert_bundle(PbrBundle::default());

        println!("Spawning pbr bundle");
        
        if let Some(mesh) = get_mesh(properties) {
            println!("Inserting mesh handle");
            world.resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                let handle = meshes.add(mesh);
                world.entity_mut(entity).insert(handle);
            });
        }

        // let (color,path) = get_material_props(properties);

        // if color.is_none() && path.is_none() {
        //     return;
        // }

        // let tex = match path {
        //     Some(path) => {
        //         let server = world.get_resource::<AssetServer>().unwrap();
        //         Some(server.load(path.as_str()))
        //     }
        //     None => None
        // };

        // world.resource_scope(|world, mut materials: Mut<Assets<StandardMaterial>>| {
        //     println!("Inserting material");
        //     let mat = StandardMaterial {
        //         base_color: color.cloned().unwrap_or_default(),
        //         base_color_texture: tex,
        //         ..Default::default()
        //     };
        //     let mat = materials.add(mat);
        //     world.entity_mut(entity).insert(mat);
        // });
    }
}

fn get_mesh(properties: Option<&DynamicStruct>) -> Option<Mesh> {
    if let Some(props) = properties {
        if let Ok(shape) = props.try_get::<String>("shape") {
            println!("Found shape {}", shape);
            return match shape.as_str() {
                "Plane" => {
                    let size = *props.try_get::<f32>("size").unwrap_or(&1.0);
                    Some(Mesh::from(shape::Plane { size }))
                }
                "Cube" => {
                    println!("Setting mesh to cube");
                    let size = *props.try_get::<f32>("size").unwrap_or(&1.0);
                    Some(Mesh::from(shape::Cube { size }))
                }
                "Quad" => {
                    let size = *props.try_get::<Vec2>("size").unwrap_or(&Vec2::ONE);
                    let flip = *props.try_get::<bool>("flip").unwrap_or(&false);
                    Some(Mesh::from(shape::Quad { size, flip }))
                }
                _ => None,
            };
        }
    }
    None
}

#[derive(Default)]
pub struct PerspectiveCameraBundleProcessor;
impl PrefabProcessor for PerspectiveCameraBundleProcessor {
    fn key(&self) -> &str {
        PERSPECTIVE_CAMERA_BUNDLE_PROCESSOR_KEY
    }

    fn process_prefab(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        println!("Spawning camera");
        world.entity_mut(entity).insert_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0,0.0,10.0),
            ..PerspectiveCameraBundle::new_3d()
        });
        drop(&properties);
    }
}

/// Implement a [PrefabProcessor] which does nothing but insert a bundle.
/// 
/// ### Arguments
/// 
/// - `type_name` : The type name for your processor.
/// - `key` : The key name for your processor. This is how you refer to your processor from the *.prefab* file.
/// - `bundle` : The function which returns the bundle instance that will be inserted.
/// 
/// ### Example
/// 
/// ```ignore
/// impl_bundle_processor!(
/// OrthographicCameraBundleProcessor,
/// OrthographicCameraBundle,
/// OrthographicCameraBundle::new_2d()
/// );
/// ```
#[macro_export]
macro_rules! impl_bundle_processor {
    ($type_name:ident, $key:ident, $bundle:expr) => {
        #[derive(Default)]
        pub struct $type_name;
        impl PrefabProcessor for $type_name {
            fn key(&self) -> &str {
                stringify!($key)
            }

            fn process_prefab(
                &self,
                properties: Option<&DynamicStruct>,
                world: &mut World,
                entity: Entity,
            ) {
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

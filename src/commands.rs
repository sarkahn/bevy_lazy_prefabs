use std::sync::Arc;

use bevy::{prelude::*, reflect::DynamicStruct};

use crate::{dynamic_cast::*, PrefabRegistry, prefab::{PrefabBuildStep, PrefabComponent}};

/// A prefab command for handling more complex prefab entity initialization.
/// 
/// A prefab command can perform complex initialization on prefab entities that can't
/// reasonably be handled from a text file. This includes things like inserting bundles,
/// loading handles for meshes and materials, and initializing any other kind of asset or
/// property that requires external data.
pub trait PrefabCommand {
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
    fn run(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity);
    
    /// The key for this processor. This is the name you refer to the processor by
    /// from your *.prefab* file.
    fn key(&self) -> &str;
}

/// Adds a [ColorMaterial] to the entity.
///
/// ### Optional Properties:
///
/// - `color` - The color for the material.
/// - `texture_path` - The path to the texture for the material.
#[derive(Default)]
pub struct AddColorMaterial;
impl PrefabCommand for AddColorMaterial {
    fn run(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        let (color, path) = get_material_props(properties);

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
            let handle = get_material(world, (color, path)).unwrap_or_default();

            world.entity_mut(entity).insert(handle);
        }
    }

    fn key(&self) -> &str {
        "AddColorMaterial"
    }
}

fn get_material_props(properties: Option<&DynamicStruct>) -> (Option<&Color>, Option<&String>) {
    if let Some(properties) = properties {
        let color = properties.try_get::<Color>("color").ok();
        let tex_path = properties.try_get::<String>("texture_path").ok();

        return (color, tex_path);
    }
    (None, None)
}

fn get_material(
    world: &mut World,
    material_props: (Option<&Color>, Option<&String>),
) -> Option<Handle<ColorMaterial>> {
    let (col, path) = material_props;

    let tex: Option<Handle<Texture>> = match path {
        Some(path) => {
            let server = world.get_resource::<AssetServer>().unwrap();
            Some(server.load(path.as_str()).clone())
        }
        None => None,
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

/// Loads a prefab and applies it's components/commands to the entity.
///
/// ### Required Property:
///
/// - `name` - The name of the prefab, including the extension.
#[derive(Default)]
pub struct LoadPrefab;
impl PrefabCommand for LoadPrefab {
    fn run(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        if let Some(props) = properties {
            if let Ok(name) = props.try_get::<String>("name") {
                world.resource_scope(|world, mut reg: Mut<PrefabRegistry>| {

                    let prefab = reg.load(name.as_str()).unwrap().clone();

                    for step in prefab.steps.iter() {
                        match step {
                            crate::prefab::PrefabBuildStep::AddComponent(comp) => {
                                let reg = &reg
                                    .get_type_data(comp.type_name.as_str())
                                    .unwrap()
                                    .registration;
                                let type_id = reg.type_id().clone();
                    
                                let reflect = match reg.data::<ReflectComponent>() {
                                    Some(reflect) => reflect,
                                    None => panic!("Error reading reflect data. Does the type {} have the '#[reflect(Component)]' attribute?", reg.short_name()),
                                }.clone();
                        
                                if world.entity(entity).contains_type_id(type_id) {
                                    reflect.apply_component(world, entity, &*comp.reflect);
                                } else {
                                    reflect.add_component(world, entity, &*comp.reflect);
                                }
                            },
                            crate::prefab::PrefabBuildStep::RunCommand(data) => {
                                let cmd = reg.get_command(data.name.as_str()).unwrap();

                                cmd.run(data.properties.as_ref(), world, entity);
                            },
                        }
                    }
                });
            }
        }
    }

    fn key(&self) -> &str {
        "LoadPrefab"
    }
}

/// Inserts a sprite bundle.
///
/// ### Optional Properties:
///
/// - `color` - The color for the material.
/// - `texture_path` - The path to the texture for the material.
#[derive(Default)]
pub struct InsertSpriteBundle;
impl PrefabCommand for InsertSpriteBundle {
    fn run(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        let (color,path) = get_material_props(properties);
        let mat = get_material(world, (color,path));

        let mut entity = world.entity_mut(entity);
        entity.insert_bundle(SpriteBundle {
            material: mat.unwrap_or_default(),
            ..Default::default()
        });
    }

    fn key(&self) -> &str {
        "InsertSpriteBundle"
    }
}
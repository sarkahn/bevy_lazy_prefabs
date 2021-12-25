//! Commands used for handling more complex prefab entity initialization, such as bundles, materials, and meshes.

use bevy::{prelude::*, reflect::DynamicStruct};

use crate::{dynamic_cast::*, PrefabRegistry};

/// A build command for handling more complex prefab entity initialization.
///
/// A build command can perform complex initialization on prefab entities that can't
/// reasonably be handled from a text file. This includes things like inserting bundles,
/// loading handles for meshes and materials, and initializing any other kind of asset or
/// property that requires external data.
pub trait BuildPrefabCommand {
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

    /// The key for this command. This is the name you refer to the command by
    /// from your *.prefab* file.
    fn key(&self) -> &str;
}

/// Sets [ColorMaterial] values on the entity.
///
/// ### Optional Properties:
///
/// - `color` - The color for the material.
/// - `texture_path` - The path to the texture for the material.
#[derive(Default)]
pub struct SetColorMaterial;
impl BuildPrefabCommand for SetColorMaterial {
    fn run(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        let (color, path) = get_material_props(properties);

        if let Some(existing_mat) = world.get_mut::<Handle<ColorMaterial>>(entity) {
            let existing_mat = existing_mat.clone_weak();
            world.resource_scope(|world, mut materials: Mut<Assets<ColorMaterial>>| {
                let mat = materials.get_mut(existing_mat).unwrap();

                if let Some(col) = color {
                    mat.color = *col;
                }
                if let Some(path) = path {
                    let server = world.get_resource::<AssetServer>().unwrap();
                    let tex: Handle<Texture> = server.load(path.as_str());
                    mat.texture = Some(tex);
                }
            });
        }
    }

    fn key(&self) -> &str {
        "SetColorMaterial"
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

fn get_color_material(
    world: &mut World,
    material_props: (Option<&Color>, Option<&String>),
) -> Option<Handle<ColorMaterial>> {
    let (col, path) = material_props;

    let tex: Option<Handle<Texture>> = match path {
        Some(path) => {
            let server = world.get_resource::<AssetServer>().unwrap();
            Some(server.load(path.as_str()))
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
    Some(materials.add(mat))
}

/// Loads a prefab and performs it's build steps on the entity.
///
/// ### Required Property:
///
/// - `name` - The name of the prefab, including the extension.
#[derive(Default)]
pub struct LoadPrefab;
impl BuildPrefabCommand for LoadPrefab {
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
                                let type_id = reg.type_id();
                                let reflect = match reg.data::<ReflectComponent>() {
                                    Some(reflect) => reflect,
                                    None => panic!("Error reading reflect data. 
                                        Does the type {} have the '#[reflect(Component)]' attribute?", reg.short_name()),
                                }.clone();
                                if world.entity(entity).contains_type_id(type_id) {
                                    reflect.apply_component(world, entity, &*comp.reflect);
                                } else {
                                    reflect.add_component(world, entity, &*comp.reflect);
                                }
                            },
                            crate::prefab::PrefabBuildStep::RunCommand(data) => {
                                let cmd = reg.get_build_command(data.name.as_str()).unwrap();

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

/// Inserts a [SpriteBundle].
///
/// ### Optional Properties:
///
/// - `color` - The color for the material.
/// - `texture_path` - The path to the texture for the material.
#[derive(Default)]
pub struct InsertSpriteBundle;
impl BuildPrefabCommand for InsertSpriteBundle {
    fn run(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        let (color, path) = get_material_props(properties);
        let mat = get_color_material(world, (color, path));

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


/// Inserts a [PbrBundle].
/// 
/// ### Optional Properties:
///
/// - `shape` - The shape to use for the mesh. Accepts `shape::Cube`, `shape::Plane` or `shape::Quad`.
/// - `size` - For a Cube or Plane the size is a single `f32`. For a Quad the size is a `Vec2`.
/// - `flip` - A `bool` that determines the texture coordinates on a [shape::Quad].
#[derive(Default)]
pub struct InsertPbrBundle;
impl BuildPrefabCommand for InsertPbrBundle {
    fn run(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        let mut bundle = PbrBundle::default();

        if let Some(properties) = properties {
            if let Some(mesh) = get_mesh(properties) {
                world.resource_scope(|_, mut meshes: Mut<Assets<Mesh>>| {
                    let handle = meshes.add(mesh);
                    bundle.mesh = handle;
                });
            }

            if let Ok(color) = properties.try_get::<Color>("color") {
                world.resource_scope(|_, mut materials: Mut<Assets<StandardMaterial>>| {
                    let mat = materials.add(StandardMaterial::from(*color));
                    bundle.material = mat;
                });
            }
        }

        world.entity_mut(entity).insert_bundle(bundle);
    }

    fn key(&self) -> &str {
        "InsertPbrBundle"
    }
}

fn get_mesh(props: &DynamicStruct) -> Option<Mesh> {
    if let Ok(shape) = props.try_get::<String>("shape") {
        return match shape.as_str() {
            "Plane" => {
                let size = *props.try_get::<f32>("size").unwrap_or(&1.0);
                Some(Mesh::from(shape::Plane { size }))
            }
            "Cube" => {
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
    None
}

/// Inserts an [OrthographicCameraBundle].
/// 
/// # Optional Property
/// 
/// - `scale` - Determines the scale of the orthographic projection.
#[derive(Default)]
pub struct InsertOrthographicCameraBundle;
impl BuildPrefabCommand for InsertOrthographicCameraBundle {
    fn run(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        let mut bundle = OrthographicCameraBundle::new_2d();

        if let Some(props) = properties {
            if let Ok(scale) = props.try_get::<f32>("scale") {
                bundle.orthographic_projection.scale = *scale;
            }
        }

        world.entity_mut(entity).insert_bundle(bundle);
    }

    fn key(&self) -> &str {
        "InsertOrthographicCameraBundle"
    }
}


/// Inserts a [PerspectiveCameraBundle].
/// 
/// # Optional Properties
/// 
/// - `position` - A `Vec3` that sets the intial position of the camera.
/// - `looking_at` - A `Vec3` that determins where the camera is initially looking.
#[derive(Default)]
pub struct InsertPerspectiveCameraBundle;
impl BuildPrefabCommand for InsertPerspectiveCameraBundle {
    fn run(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {
        let mut bundle = PerspectiveCameraBundle::new_3d();

        if let Some(props) = properties {
            if let Ok(position) = props.try_get::<Vec3>("position") {
                bundle.transform.translation = *position;
            }

            if let Ok(looking_at) = props.try_get::<Vec3>("looking_at") {
                bundle.transform = bundle.transform.looking_at(*looking_at, Vec3::Y);
            }
        }

        world.entity_mut(entity).insert_bundle(bundle);
    }

    fn key(&self) -> &str {
        "InsertPerspectiveCameraBundle"
    }
}
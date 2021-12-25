use bevy::{prelude::*, reflect::DynamicStruct};

use crate::dynamic_cast::*;

pub trait PrefabCommand {
    fn run(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity);
    fn key(&self) -> &str;
}

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

pub struct LoadPrefab;
impl PrefabCommand for LoadPrefab {
    fn run(&self, properties: Option<&DynamicStruct>, world: &mut World, entity: Entity) {}

    fn key(&self) -> &str {
        todo!()
    }
}

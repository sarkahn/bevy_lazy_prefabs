use std::sync::Arc;

use bevy::{ecs::system::Command, prelude::*};

use crate::{
    prefab::{Prefab, PrefabComponent, PrefabLoad, PrefabProcessorData},
    registry::PrefabRegistry
};

/// Adds the `spawn_prefab` option to bevy [Commands].
pub trait SpawnPrefabCommands {
    /// Spawn a prefab from a ".prefab" file.
    /// 
    /// The prefab name should include the file extension. Prefabs files are loaded
    /// from the *assets/prefabs* directory.
    /// 
    /// # Example: 
    /// 
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_lazy_prefabs::*;
    /// 
    /// fn setup(mut commands: Commands) {
    ///   commands.spawn_prefab("sprite.prefab");
    ///   commands.spawn_prefab("camera.prefab");
    ///  }
    /// ```
    fn spawn_prefab(&mut self, prefab_name: &str);
}

struct SpawnPrefab {
    prefab_name: String,
}

impl Command for SpawnPrefab {
    fn write(self: Box<Self>, world: &mut World) {
        world.resource_scope(|world, mut reg: Mut<PrefabRegistry>| {
            reg.load(self.prefab_name.as_str()).unwrap();

            let prefab = reg
                .get_prefab(self.prefab_name.as_str())
                .unwrap()
                .clone();

            let entity = world.spawn().id();
            load_prefab(prefab, &mut reg, world, entity);
        });
    }
}

impl<'w> SpawnPrefabCommands for Commands<'w> {
    fn spawn_prefab(&mut self, prefab_name: &str) {
        self.add(SpawnPrefab {
            prefab_name: prefab_name.to_string(),
        });
    }
}

pub struct SpawnPrefabFromString {
    prefab_name: String,
    input: String,
}

impl Command for SpawnPrefabFromString {
    fn write(self: Box<Self>, world: &mut World) {
        world.resource_scope(|world, mut reg: Mut<PrefabRegistry>| {
            let prefab = reg.load_from_string(self.prefab_name.as_ref(), self.input.as_ref()).unwrap();

            let entity = world.spawn().id();
            load_prefab(prefab.clone(), &mut reg, world, entity);
        });
    }
}

pub(crate) enum PrefabCommand {
    AddComponent(PrefabComponent),
    Processor(PrefabProcessorData),
    LoadPrefab(PrefabLoad),
}

fn load_prefab(
    prefab: Arc<Prefab>,
    registry: &mut PrefabRegistry,
    world: &mut World,
    entity: Entity,
) {
    for command in prefab.commands() {
        process_command(command, world, registry, entity);
    }
}

fn process_command(
    command: &PrefabCommand,
    world: &mut World,
    registry: &mut PrefabRegistry,
    entity: Entity,
) {
    match command {
        PrefabCommand::AddComponent(comp) => {
            let type_id = comp.type_id();
            if world.entity(entity).contains_type_id(type_id) {
                comp.reflect().apply_component(world, entity, comp.root());
            } else {
                comp.reflect().add_component(world, entity, comp.root());
            }
        }
        PrefabCommand::Processor(proc) => {
            proc.processor()
                .process_prefab(proc.properties(), world, entity);
        }
        PrefabCommand::LoadPrefab(load) => {
            registry.load(load.path()).unwrap();
            let prefab = registry.get_prefab(load.path()).unwrap().clone();

            load_prefab(prefab, registry, world, entity);
        }
    }
}
use std::sync::Arc;

use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
};

use crate::{
    prefab::{Prefab, PrefabCommandData, PrefabComponent},
    PrefabRegistry,
};

pub trait SpawnPrefabCommands {
    /// Apply [Prefab] components and commands to an entity.
    ///
    /// Prefabs can be loaded from the [PrefabRegistry].
    fn insert_prefab(&mut self, prefab: &Prefab) -> &mut Self;
}

impl SpawnPrefabCommands for EntityCommands<'_, '_> {
    fn insert_prefab(&mut self, prefab: &Prefab) -> &mut Self {
        let id = self.id();
        for step in prefab.steps.iter() {
            match step {
                crate::prefab::PrefabBuildStep::AddComponent(comp) => {
                    self.commands().add(AddComponentCommand {
                        entity: id,
                        component: comp.clone(),
                    });
                }
                crate::prefab::PrefabBuildStep::RunCommand(command) => {
                    self.commands().add(PrefabProcessCommand {
                        entity: id,
                        data: command.clone(),
                    });
                }
            }
        }

        self
    }
}

struct AddComponentCommand {
    entity: Entity,
    component: Arc<PrefabComponent>,
}

impl Command for AddComponentCommand {
    fn write(self: Box<Self>, world: &mut World) {
        let entity = self.entity;
        let component = self.component;

        let registry = world.get_resource::<PrefabRegistry>().unwrap();

        let reg = &registry
            .get_type_data(component.type_name.as_str())
            .unwrap()
            .registration;
        let type_id = reg.type_id();

        let reflect = match reg.data::<ReflectComponent>() {
            Some(reflect) => reflect,
            None => panic!("Error reading reflect data. Does the type {} have the '#[reflect(Component)]' attribute?", reg.short_name()),
        }.clone();

        if world.entity(entity).contains_type_id(type_id) {
            reflect.apply_component(world, entity, &*component.reflect);
        } else {
            reflect.add_component(world, entity, &*component.reflect);
        }
    }
}

pub struct PrefabProcessCommand {
    entity: Entity,
    data: Arc<PrefabCommandData>,
}

impl Command for PrefabProcessCommand {
    fn write(self: Box<Self>, world: &mut World) {
        let entity = self.entity;
        let data = self.data;
        let command_name = data.name.as_str();

        let reg = world.get_resource::<PrefabRegistry>().unwrap();
        let command = reg.get_build_command(command_name).unwrap_or_else(|| {
            panic!(
                "Error performing prefab command {}. Was it registered in the PrefabRegistry?",
                command_name
            )
        });

        let command = command.clone();
        command.run(data.properties.as_ref(), world, entity);
    }
}

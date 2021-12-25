use std::sync::Arc;

use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
};

use crate::{
    prefab::{Prefab, PrefabCommandData, PrefabComponent},
    PrefabRegistry,
};

pub trait SpawnPrefabCommands<'a> {
    fn spawn_prefab<'b>(
        &'b mut self,
        prefab: Handle<Prefab>,
        registry: &PrefabRegistry,
    ) -> EntityCommands<'a, 'b>;
}
impl<'a> SpawnPrefabCommands<'a> for Commands<'a> {
    fn spawn_prefab<'b>(
        &'b mut self,
        prefab: Handle<Prefab>,
        registry: &PrefabRegistry,
    ) -> EntityCommands<'a, 'b> {
        let mut entity = self.spawn();
        let id = entity.id();

        entity.insert(LoadPrefabCommand {
            entity: id,
            handle: prefab,
        });

        entity
    }
}

struct LoadPrefabCommand {
    entity: Entity,
    handle: Handle<Prefab>,
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
            None => panic!("Error reading reflect data. Does the  type {} have the '#[reflect(Component)]' attribute?", reg.short_name()),
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
    data: PrefabCommandData,
}

impl Command for PrefabProcessCommand {
    fn write(self: Box<Self>, world: &mut World) {
        let entity = self.entity;
        let data = self.data;
        let command_name = data.command_name.as_str();

        world.resource_scope(|world, registry: Mut<PrefabRegistry>| {
            let command = registry.get_command(command_name).unwrap().clone();

            command.run(data.properties.as_ref(), world, entity);
        });
    }
}

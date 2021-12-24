use std::any::Any;

use bevy::ecs::system::Command;

use bevy::prelude::*;
use bevy::reflect::DynamicStruct;
use super::registry::PrefabRegistry;

pub struct PrefabAddComponent {
    entity: Entity,
    type_name: String,
    data: Box<dyn Reflect>, 
}

impl Command for PrefabAddComponent {
    fn write(self: Box<Self>, world: &mut World) {
        let entity = self.entity;
        let comp = self.data;

        let registry = world.get_resource::<PrefabRegistry>().unwrap();

        let t = registry.get_type_data(self.type_name.as_str()).unwrap();
        let type_id = t.type_id();

        let reflect = match t.data::<ReflectComponent>() {
            Some(reflect) => reflect,
            None => panic!("Error reading reflect data. Does the  type {} have the '#[reflect(Component)]' attribute?", t.short_name()),
        }.clone();

        if world.entity(entity).contains_type_id(type_id) {
            reflect.apply_component(world, entity, &*comp);
        } else {
            reflect.add_component(world, entity, &*comp);
        }
    }
}

pub struct PrefabProcessCommand {
    entity: Entity,
    command_name: String,
    properties: Option<DynamicStruct>,
}

impl Command for PrefabProcessCommand {
    fn write(self: Box<Self>, world: &mut World) {
        let entity = self.entity;
        let command_name = self.command_name.as_str();

        world.resource_scope(|world, registry: Mut<PrefabRegistry>| {
            let command = registry.get_command(command_name).unwrap().clone();

            command.run(self.properties.as_ref(), world, entity);
        });
    }
}

/// A command for loading a nested prefab. This means all the components/commands of
/// another prefab will be applied to an existing entity.
pub struct LoadNestedPrefabCommand {
    pub entity: Entity,
    pub prefab_name: String,
}

impl Command for LoadNestedPrefabCommand {
    fn write(self: Box<Self>, world: &mut World) {
        todo!()
    }
}
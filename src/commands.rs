use bevy::{prelude::*, ecs::system::Command, reflect::{DynamicStruct, TypeRegistry}};

use crate::{registry::PrefabRegistry, prefab::Prefab};

pub struct SpawnPrefab {
    prefab_name: String,
}

impl Command for SpawnPrefab {
    fn write(self: Box<Self>, world: &mut World) {          
        let entity = world.spawn().id();

        let mut reg = world.get_resource::<PrefabRegistry>().unwrap().clone();
        let prefab = reg.load(self.prefab_name.as_str()).unwrap().clone();

        for component in prefab.components() {
            println!("Adding {}", component.name());
            let reflect = reg.reflect_component(&component.name()).unwrap();
            let root = &**component.root();
            reflect.add_component(world, entity, root);
        }
    }

}

pub trait SpawnPrefabCommands {
    fn spawn_prefab(&mut self, prefab_name: &str);
}

impl<'w> SpawnPrefabCommands for Commands<'w> {
    fn spawn_prefab(&mut self, prefab_name: &str) {
        self.add(SpawnPrefab {
            prefab_name: prefab_name.to_string()
        });
    }
}
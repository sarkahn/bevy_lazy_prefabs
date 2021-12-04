use bevy::{ecs::system::Command, prelude::*};

use crate::registry::PrefabRegistryArc;

struct SpawnPrefab {
    prefab_name: String,
}

impl Command for SpawnPrefab {
    fn write(self: Box<Self>, world: &mut World) {
        let entity = world.spawn().id();

        {
            let reg = world
                .get_resource_mut::<PrefabRegistryArc>()
                .unwrap()
                .clone();
            let mut reg = reg.write();
            reg.load(self.prefab_name.as_str()).unwrap();
        }

        let reg = world.get_resource::<PrefabRegistryArc>().unwrap().clone();
        let reg = reg.read();

        let prefab = reg.get_prefab(self.prefab_name.as_str()).unwrap();

        for component in prefab.components() {
            let reflect = reg.reflect_component(component.name()).unwrap();

            reflect.add_component(world, entity, component.root());
        }
    }
}

pub trait SpawnPrefabCommands {
    /// Spawn a prefab from a ".prefab" file. 
    fn spawn_prefab(&mut self, prefab_name: &str);
}

impl<'w> SpawnPrefabCommands for Commands<'w> {
    fn spawn_prefab(&mut self, prefab_name: &str) {
        self.add(SpawnPrefab {
            prefab_name: prefab_name.to_string(),
        });
    }
}
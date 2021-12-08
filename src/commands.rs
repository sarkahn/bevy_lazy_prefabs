use bevy::{ecs::system::Command, prelude::*};

use crate::{ 
    registry::PrefabRegistry
};

struct SpawnPrefab {
    prefab_name: String,
}

impl Command for SpawnPrefab {
    fn write(self: Box<Self>, world: &mut World) {

        world.resource_scope(|world, mut reg: Mut<PrefabRegistry>| {
            // Load the prefab if it's not already loaded
            reg.load(self.prefab_name.as_str()).unwrap();

            let prefab = reg.get_prefab(self.prefab_name.as_str()).unwrap();

            let mut entity = world.spawn();

            // First insert bundles
            if let Some(bundles) = prefab.bundles() {
                for bundle in bundles {
                    reg.get_bundle_loader(bundle.name()).add_bundle(&mut entity);
                }
            }

            // Then our material data
            if let Some(mat) = prefab.material() {
                entity.insert(mat.clone());
            }

            let entity = entity.id();

            // Finally the individual components
            for component in prefab.components() {
                let reflect = reg.reflect_component(component.name()).unwrap();
        
                reflect.add_component(world, entity, component.root());
            }
        });
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
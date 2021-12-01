use bevy::{prelude::*, ecs::system::Command};

use crate::{registry::PrefabRegistry, prefab::Prefab};

pub struct SpawnPrefab {
    prefab: Handle<Prefab>,
}

impl Command for SpawnPrefab {
    fn write(self: Box<Self>, world: &mut World) {
                     
        let assets = world.get_resource::<Assets<Prefab>>().unwrap();
        let prefab = assets.get(self.prefab).unwrap();

        let reg = world.get_resource::<PrefabRegistry>()
            .expect("Error spawning prefab - prefab registry hasn't been added as a resource.
                     Has LazyPrefabPlugin been added?");

        

        for component in prefab.components.iter() {

        }
    }
}
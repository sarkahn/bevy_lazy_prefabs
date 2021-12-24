use bevy::{prelude::*, ecs::system::EntityCommands};

use super::{prefab::Prefab, prefab_builder::LoadNestedPrefabCommand};

pub trait SpawnPrefabCommands<'a> {
    fn spawn_prefab<'b>( &'b mut self, 
        Prefab: Prefab, 
    ) -> EntityCommands<'a, 'b>;
}
impl<'a> SpawnPrefabCommands<'a> for Commands<'a> {
    fn spawn_prefab<'b>(
        &'b mut self, 
        prefab: Prefab,
    ) -> EntityCommands<'a, 'b> {
        let mut entity = self.spawn();
        let id = entity.id();



        entity
    }
}

fn LoadPrefab(commands: &mut Commands, entity: Entity, prefab_name: &str) {

}


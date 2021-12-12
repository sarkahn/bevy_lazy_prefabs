use std::sync::Arc;

use bevy::{ecs::{system::Command}, prelude::*};

use crate::{registry::PrefabRegistry, prefab::{Prefab, PrefabComponent, PrefabProcessorData, PrefabLoad}, PrefabProcessor};

struct SpawnPrefab {
    prefab_name: String,
}

impl Command for SpawnPrefab {
    fn write(self: Box<Self>, world: &mut World) {
        world.resource_scope(|world, mut reg: Mut<PrefabRegistry>| {

            reg.load(self.prefab_name.as_str()).unwrap();

            let prefab = reg.get_arc_prefab(self.prefab_name.as_str()).unwrap().clone();

            let entity = world.spawn().id();
            load_prefab(prefab, &mut reg, world, entity);

        });
    }
}

fn load_prefab(prefab: Arc<Prefab>, registry: &mut PrefabRegistry, world: &mut World, entity: Entity) {
    for command in prefab.commands() {
        process_command(command, world, registry, entity);
    }
}


/// Adds the `spawn_prefab` option to bevy [Commands].
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

pub(crate) enum PrefabCommand {
    AddComponent(PrefabComponent),
    Processor(PrefabProcessorData),
    LoadPrefab(PrefabLoad),
}

fn process_command(command: &PrefabCommand, world: &mut World, registry: &mut PrefabRegistry, entity: Entity) {
    match command {
        PrefabCommand::AddComponent(comp) => {
            let type_id = comp.type_id();
            if world
            .entity(entity)
            .contains_type_id(type_id) 
            {
                comp.reflect().apply_component(world, entity, comp.root());
            } else {
                comp.reflect().add_component(world, entity, comp.root());
            }

        },
        PrefabCommand::Processor(proc) => {
            proc.processor().process_prefab(proc.properties(), world, entity);
        },
        PrefabCommand::LoadPrefab(load) => {
            registry.load(load.path()).unwrap();
            let prefab = registry.get_arc_prefab(load.path()).unwrap().clone();

            load_prefab(prefab, registry, world, entity);
        },
    }
}

#[cfg(test)]
mod tests {
    use bevy::{prelude::*, reflect::{GetTypeRegistration, DynamicStruct, TypeRegistration}};

    #[derive(Reflect, Default)]
    #[reflect(Component)]
    struct MyStruct {
        x: i32,
        y: i32,
    }

    fn reflect<T: Reflect + GetTypeRegistration>() -> TypeRegistration {
        T::get_type_registration()
    }
    
    #[test]
    fn test() {
        let mut a = MyStruct {
            x: 0,
            y: 0
        };

        let r = reflect::<MyStruct>();
        let reflect = r.data::<ReflectComponent>().unwrap();
    
        let t = MyStruct::get_type_registration();

        let reflect = t.data::<ReflectComponent>().unwrap();
        

        a.x = 15;

        let mut world = World::default();

        let mut entity = world.spawn();

        entity.insert(a);

        let id = entity.id();
    
        let c = DynamicStruct::default();
        reflect.apply_component(&mut world, id, &c);

        let a = world.get::<MyStruct>(id).unwrap();

        assert_eq!(15, a.x);
    }
}
use bevy::{ecs::{system::Command}, prelude::*};

use crate::{registry::PrefabRegistry, prefab::{Prefab}};

struct SpawnPrefab {
    prefab_name: String,
}

impl Command for SpawnPrefab {
    fn write(self: Box<Self>, world: &mut World) {
        world.resource_scope(|world, mut reg: Mut<PrefabRegistry>| {
            
            // Load the prefab if it's not already loaded
            reg.load(self.prefab_name.as_str()).unwrap();

            let prefab = reg.get_prefab(self.prefab_name.as_str()).unwrap();

            let entity = world.spawn().id();

            add_prefab(prefab, &reg, world, entity);            
        });
    }
}

fn add_prefab(prefab: &Prefab, reg: &PrefabRegistry, world: &mut World, entity: Entity) {
    let entity = &mut world.entity_mut(entity);
    
    if let Some(processors) = prefab.processors() {
        for data in processors {
            let processor = reg.get_processor(data.key()).unwrap_or_else(|| {
                panic!(
                    "Error spawning prefab, the processor {} hasn't been registered",
                    data.key()
                )
            });
            processor.process_prefab(data.properties(), entity);
        }
    }

    let id = entity.id();

    if let Some(loaders) = prefab.loaders() {
        for load in loaders {
            let nested = reg.get_prefab(load.path()).unwrap();
            add_prefab(nested, reg, world, id);
        }
    }

    for component in prefab.components() {
        let reflect = reg.reflect_component(component.name()).unwrap();
        let t = reg.type_info(component.name()).unwrap();

        if world
            .entity(id)
            .contains_type_id(t.registration.type_id()) 
        {
            reflect.apply_component(world, id, component.root());
        } else {
            reflect.add_component(world, id, component.root());
        }

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
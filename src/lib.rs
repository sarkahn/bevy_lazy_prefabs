
mod prefab;
mod parse;
mod registry;
mod prefab_reflect;
mod commands;
mod dynamic_cast;
mod bundle;

use std::borrow::Borrow;
use std::sync::Arc;

use bevy::prelude::*;
use bevy::reflect::{TypeRegistryArc,TypeRegistry, DynamicStruct};

pub use registry::{ 
    PrefabRegistry as PrefabRegistryInternal, 
    PrefabRegistryArc as PrefabRegistry 
};
pub use commands::SpawnPrefabCommands;

pub struct LazyPrefabsPlugin;
impl Plugin for LazyPrefabsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<registry::PrefabRegistryArc>();
    }
}


#[derive(Reflect)]
struct Test {
    x: i32,
}

fn reflect_example(
    world: &mut World,

) {
    let registry = world.get_resource::<TypeRegistry>().unwrap().clone();
    let registry = registry.read();
    
    let registration = registry.get_with_name("aaa")
        .expect("Unregistered type" );

    let reflect_component = registration.data::<ReflectComponent>()
        .expect( "Unregistered component");

    let mut struc = DynamicStruct::default();
    struc.insert("x", 15i32);

    let entity = world.spawn().id();
    reflect_component.add_component(world, entity, &*Box::new(struc));
}

fn reflect_example2(
    world: &mut World
) {
    let entity = world.spawn().id();
    
    {
        let registry = world.get_resource_mut::<PrefabRegistry>().unwrap();
        let mut registry = registry.write();
        registry.load("hi").unwrap();
    }

    let registry = world.get_resource::<PrefabRegistry>().unwrap().clone();
    let registry = registry.read();

    let prefab = registry.get_prefab("hi").unwrap();

    let component = &**prefab.component_from_index(0).root();

    let reflect = registry.reflect_component("Hi").unwrap();

    reflect.add_component(world, entity, component);
}

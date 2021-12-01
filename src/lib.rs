
mod prefab;
mod parse;
mod registry;
mod prefab_reflect;
mod commands;
mod dynamic_cast;

use bevy::prelude::*;
use bevy::reflect::{TypeRegistryArc};

pub use registry::PrefabRegistry;

pub struct LazyPrefabsPlugin;
impl Plugin for LazyPrefabsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<registry::PrefabRegistry>();
    }
}


fn reflect_example(
    world: &mut World,

) {
    let registry = world.get_resource::<TypeRegistryArc>().unwrap().clone();
    let registry = registry.read();
    
    let registration = registry.get_with_name("aaa")
        .expect("Unregistered type" );

    let reflect_component = registration.data::<ReflectComponent>()
        .expect( "Unregistered component");

    let component: Box<dyn Reflect> = Box::new(Transform::default());

    let entity = world.spawn().id();
    reflect_component.add_component(world, entity, &*component);
}
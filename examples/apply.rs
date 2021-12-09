use bevy::{prelude::*, reflect::{GetTypeRegistration, TypeRegistry, DynamicStruct}};

#[derive(Reflect, Default, Debug)]
#[reflect(Component)]
struct AStruct {
    x: i32,
    y: i32,
}

fn go(
    world: &mut World
) {

    let a = AStruct {
        x: 10, y: 15
    };

    let mut entity = world.spawn();
    entity.insert(a);

    let t = AStruct::get_type_registration();
    let r = t.data::<ReflectComponent>().unwrap();

    let a = DynamicStruct::default();

    let id = entity.id();

    r.apply_component(world, id, &a);

    let a = world.get::<AStruct>(id).unwrap();

    println!("A {:#?}", a);

}

fn main() {
    App::build()
    .add_plugins(DefaultPlugins)
    .register_type::<AStruct>()
    .add_startup_system(go.exclusive_system())
    .run();
}



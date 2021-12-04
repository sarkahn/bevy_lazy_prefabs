use bevy::{prelude::*, reflect::DynamicStruct};
use bevy_lazy_prefabs::{LazyPrefabsPlugin, PrefabRegistry};
use bevy::reflect::{TypeRegistryArc, TypeRegistry};

#[derive(Reflect, Default, Debug)]
#[reflect(Component)]
struct TestComponentA;
#[derive(Reflect, Default, Debug)]
#[reflect(Component)]
struct TestComponentB {
    x: i32,
}

fn spawn_prefab(
    world: &mut World
) {
    {
        let prefabs = world.get_resource_mut::<PrefabRegistry>().unwrap();
        let mut prefabs = prefabs.write();
        prefabs.register_component::<TestComponentA>();
        prefabs.register_component::<TestComponentB>();
    }

    let entity = world.spawn().id();

    let prefabs = world.get_resource::<PrefabRegistry>().unwrap().clone();
    let prefabs = prefabs.read();

    let mut prefab = DynamicStruct::default();
    prefab.insert("x", 35i32);
    
    let reflect = prefabs.reflect_component("TestComponentB").unwrap();
    reflect.add_component(world, entity, &prefab);
    

    let reflect = prefabs.reflect_component("TestComponentA").unwrap();
    reflect.add_component(world, entity, &prefab);
}

fn setup(
    prefabs: ResMut<PrefabRegistry>,
) {
    let mut prefabs = prefabs.write();
    prefabs.register_component::<TestComponentA>();
    prefabs.register_component::<TestComponentB>();
}

fn print_test_entites(
    input: Res<Input<KeyCode>>,
    q_test: Query<&TestComponentB>,
    q_a: Query<&TestComponentA>,
) {
    if input.just_pressed(KeyCode::Space) {
        for comp in q_test.iter() {
            println!("Found testcomponent: {:#?}", comp);
        }

        for comp in q_a.iter() {
            println!("Found testcomponent A");
        }
    }
}

fn main () {
    App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(LazyPrefabsPlugin)
    .register_type::<TestComponentA>()
    .register_type::<TestComponentB>()
    .add_startup_system(setup.system().label("Setup"))
    .add_startup_system(spawn_prefab.exclusive_system())
    .add_system(print_test_entites.system())
    .run();
}
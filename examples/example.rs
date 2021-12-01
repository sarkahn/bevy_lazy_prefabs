use bevy::prelude::*;
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
        let mut prefabs = world.get_resource_mut::<PrefabRegistry>().unwrap();
        prefabs.register_component::<TestComponentA>();
        prefabs.register_component::<TestComponentB>();
    }

    let types = world.get_resource::<TypeRegistryArc>().unwrap().clone();
    let types = types.read();

    let registration = types.get_with_short_name("TestComponentB").expect("Unregistered type");
    let reflect_component = registration.data::<ReflectComponent>().expect("Unreflected component");

    let mut prefabs = world.get_resource_mut::<PrefabRegistry>().unwrap();
    let mut proto = prefabs.instance_clone("TestComponentB").unwrap();
    let prefab = prefabs.load("test.prefab").expect("Error loading prefab");

    proto.apply(&*prefab.components[1].reflect);

    let entity = world.spawn().id();
    reflect_component.add_component(world, entity, &*proto);
}

fn setup(
    mut prefabs: ResMut<PrefabRegistry>,
    mut types: ResMut<TypeRegistry>,
) {
    prefabs.register_component::<TestComponentA>();
    prefabs.register_component::<TestComponentB>();
}

fn print_test_entites(
    input: Res<Input<KeyCode>>,
    q_test: Query<&TestComponentB>,
) {
    if input.just_pressed(KeyCode::Space) {
        for comp in q_test.iter() {
            println!("Found testcomponent: {:#?}", comp);
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
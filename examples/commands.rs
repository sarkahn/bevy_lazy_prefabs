use bevy_lazy_prefabs::*;
use bevy::prelude::*;

#[derive(Reflect, Default)]
#[reflect(Component)]
struct TestComponentA;

#[derive(Reflect, Default)]
#[reflect(Component)]
struct TestComponentB {
    x: i32
}

fn setup (
    registry: ResMut<PrefabRegistry>,
) {
    let mut registry = registry.write();
    registry.register_component::<TestComponentA>();
    registry.register_component::<TestComponentB>();
}

fn do_spawn(
    mut commands: Commands
) {
    commands.spawn_prefab("test.prefab");
}

fn query(
    input: Res<Input<KeyCode>>,
    q: Query<(&TestComponentA, &TestComponentB)>,
) {
    if input.just_pressed(KeyCode::Space) {
        println!("Running query...");
        for (_a,b) in q.iter() {
            println!("Found components! Value of b.x: {}", b.x);
        }
    }
}

fn main() {
    App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(LazyPrefabsPlugin)
    .add_startup_system(setup.system())
    .add_startup_system(do_spawn.system())
    .add_system(query.system())
    .run();
}
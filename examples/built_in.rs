use bevy::prelude::*;
use bevy_lazy_prefabs::*;

fn setup(mut registry: ResMut<PrefabRegistry>) {
    registry.register_type::<Transform>();
    registry.register_type::<Visible>();
    registry.register_type::<Draw>();
}

fn do_spawn(mut commands: Commands) {
    commands.spawn_prefab("builtin.prefab");
}

fn query(input: Res<Input<KeyCode>>, q: Query<(&Transform, &Visible)>) {
    if input.just_pressed(KeyCode::Space) {
        println!("Running query...");
        for (transform, _visible) in q.iter() {
            println!("Found components! Value of Transform: {:#?}", transform);
        }
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(plugins::LazyPrefabsMinimalPlugin)
        .add_startup_system(setup.system())
        .add_startup_system(do_spawn.system())
        .add_system(query.system())
        .run();
}

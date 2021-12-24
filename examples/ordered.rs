use bevy::prelude::*;
use bevy_lazy_prefabs::*;

#[derive(Reflect, Default)]
#[reflect(Component)]
struct Pos {
    x: i32,
}

fn setup(
    mut commands: Commands,
    mut registry: ResMut<PrefabRegistry>,
) {
    commands.spawn_prefab("ordered.prefab", &mut registry).unwrap();
}

fn check(q: Query<&Pos>) {
    for p in q.iter() {
        assert_eq!(p.x, 2);
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(LazyPrefabsPlugin)
        .register_prefab_type::<Pos>()
        .add_startup_system(setup.system())
        .add_system(check.system())
        .run();
}

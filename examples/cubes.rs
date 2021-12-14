use bevy::prelude::{*};
use bevy_lazy_prefabs::*;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn_prefab("cube.prefab");
    commands.spawn_prefab("persp_camera.prefab");

    // let mesh = Mesh::from(shape::Cube{ size: 2.0});
    // let mesh = meshes.add(mesh);

    // commands.spawn_bundle( PbrBundle {
    //     mesh,
    //     ..Default::default()
    // });

    // commands.spawn_bundle(PerspectiveCameraBundle {
    //     transform: Transform::from_xyz(0.0,0.0,10.0),
    //     ..Default::default()
    // });
}

// fn query(
//     q: Query<&Cube>,
// ) {
//     for cube in q.iter() {
//         println!("Cube: {}", cube.size);
//     }
// }


fn main() {
    App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(LazyPrefabsPlugin)
    .add_startup_system(setup.system())
    .run();
}
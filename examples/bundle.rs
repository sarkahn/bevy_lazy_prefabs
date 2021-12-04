use bevy::{prelude::*, reflect::TypeRegistry};


#[derive(Default, Reflect, Bundle)]
#[reflect(Component)]
struct ABundle {
    transform: Transform,
    color: Color,
}

fn setup(
    type_registry: Res<TypeRegistry>,
) {
    let name = "ABundle";
    let type_registry = type_registry.read();
    let res = type_registry.get_with_short_name(name);

    let _bundle = PbrBundle::default();
    let info = PbrBundle::type_info();

    for t in info {
        println!("{}", t.type_name());
    } 

    match res {
        Some(_) => {
            println!("{} was found!", name);
        },
        None => println!("{} not found", name),
    }
}

fn main() {
    App::build()
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup.system())
    .run();
}
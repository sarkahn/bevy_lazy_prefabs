use std::marker::PhantomData;
use bevy::{prelude::*, asset::{Asset, AssetDynamic}, reflect::TypeUuid, utils::HashMap};
use serde::Deserialize;

fn setup(
    mut commands: Commands,
) {
    let asset = PrefabMaterial::<ColorMaterial>::new("alien.png");
    commands.spawn_bundle(SpriteBundle::default()).insert(asset);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn main() {
    App::build()
    .add_plugins(DefaultPlugins)
    .add_system(prefab_load_material_system::<ColorMaterial>.system())
    .add_system(prefab_load_material_system::<MyMaterial>.system())
    .add_startup_system(setup.system())
    .run();
}

fn prefab_load_material_system<T: Asset + From<Handle<Texture>>> (
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut assets: ResMut<Assets<T>>,
    mut q: Query<(Entity, &mut Handle<T>, &PrefabMaterial<T>)>,
) {
    for (entity, mut handle, asset) in q.iter_mut() {
        let path = asset.path();
        let i: Handle<Texture> = asset_server.load(path);
        *handle = assets.add(i.into());
        commands.entity(entity).remove::<PrefabMaterial<T>>();
    }
}

fn add_mat(
    world: &mut World
) {

}


struct Registry {
    map: HashMap<String,Box<dyn Reflect>>,
}

impl Registry {
    pub fn register<T: Asset>(&mut self, path: &str) {
        let i = Box::new(PrefabMaterial::<T>::new(path));
        self.map.insert(path.to_string(), i);
    }
}

// pub trait DynamicPrefabMaterial {
//     fn path() -> &str;
// }

#[derive(Reflect)]
struct PrefabMaterial<T: Asset> {
    path: String,
    #[reflect(ignore)]
    phanom: PhantomData<T>,
}

impl<T: Asset> PrefabMaterial<T> {
    pub fn new(path: &str) -> Self {
        PrefabMaterial {
            path: path.to_string(),
            phanom: PhantomData::default(),
        }
    }
    pub fn path(&self) -> &str {
        self.path.as_ref()
    }
}

// impl DynamicPrefabMaterial for PrefabMaterial<ColorMaterial> {
//     fn path(&self) -> &str {
//         self.path.as_str()
//     }
// }

// struct Registry {
//     map: HashMap<String, Box<dyn DynamicPrefabMaterial>>,
// }

#[derive(Debug, TypeUuid)]
#[uuid = "97262015-ed6c-4797-a668-6524b2eaeb84"]
struct MyMaterial {
    tex: Handle<Texture>,
}

impl From<Handle<Texture>> for MyMaterial {
    fn from(tex: Handle<Texture>) -> Self {
        MyMaterial {
            tex
        }
    }
}

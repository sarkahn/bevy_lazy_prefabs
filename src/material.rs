use bevy::{prelude::*, asset::{Asset, AssetDynamic}, reflect::{TypeUuid, DynamicStruct, TypeUuidDynamic}, utils::{tracing::Subscriber, HashMap}};

use crate::{dynamic_cast::GetValue};

use derivative::*;

pub const COLOR_MATERIAL_LOADER_KEY: &str  = "ColorMaterial";

#[derive(Derivative)]
#[derivative(Debug)]
pub struct PrefabMaterial {
    texture_path: String,
    loader_key: String,
    #[derivative(Debug="ignore")]
    properties: Option<DynamicStruct>,
}

impl PrefabMaterial {
    pub fn new(texture_path: &str, loader_key: &str, properties: Option<DynamicStruct>) -> Self {
        PrefabMaterial {
            texture_path: texture_path.to_string(), 
            loader_key: loader_key.to_string(), 
            properties
        }
    }

    pub fn texture_path(&self) -> &str {
        self.texture_path.as_str()
    }

    pub fn loader_key(&self) -> &str {
        self.loader_key.as_str()
    }

    pub fn properties(&self) -> Option<&DynamicStruct> {
        self.properties.as_ref()
    }
}

impl Clone for PrefabMaterial {
    fn clone(&self) -> Self {
        Self { 
            texture_path: self.texture_path.clone(), 
            loader_key: self.loader_key.clone(), 
            properties: match &self.properties {
                Some(p) => Some(p.clone_dynamic()),
                None => None
            }
        }
    }
}

/// A trait you can implement to describe how to apply a material to an entity from a texture.
pub trait PrefabMaterialLoader: TypeUuidDynamic + Send + Sync + 'static {
    fn key(&self) -> &str;
    /// Retrieve the a boxed version of the target material.
    /// 
    /// * `properties` - Optional properties which can be populated with data relevant to
    ///   the construction of the material. These will be populated during prefab parsing
    ///   and can be retrieved during material construction from the loader.
    /// * `texture` - The texture which should be attached to the material.
    fn get_asset(properties: Option<&DynamicStruct>, texture: Handle<Texture>) -> Box::<dyn AssetDynamic>;
}

fn load_prefab_material<L: PrefabMaterialLoader, T: Asset + AssetDynamic> (
    mut commands: Commands,
    server: Res<AssetServer>,
    mut assets: ResMut<Assets<T>>,
    mut q: Query<(Entity, &mut Handle<T>, &PrefabMaterial)>,
) {
    for (e, mut handle, prefab_mat) in q.iter_mut() {
        let tex: Handle<Texture> = server.load(prefab_mat.texture_path.as_str());

        let asset = L::get_asset(prefab_mat.properties.as_ref(), tex);

        let cast: Box<T> = match asset.downcast() {
            Ok(res) => res,
            Err(_) => {
                panic!("Error loading prefab material, could not cast AssetDynamic to {}:", 
                std::any::type_name::<T>());
            }
        };

        *handle = assets.add(*cast);

        commands.entity(e).remove::<PrefabMaterial>();
    }
}

#[derive(Default, Debug, TypeUuid)]
#[uuid = "28af0b4a-2ba0-49d2-af61-61ad2fec467c"]
pub struct PrefabColorMaterialLoader;

impl PrefabMaterialLoader for PrefabColorMaterialLoader {
    fn get_asset(properties: Option<&DynamicStruct>, tex: Handle<Texture>) -> Box::<dyn AssetDynamic> {
        let col;

        if let Some(properties) = properties {
            col = match properties.try_get::<Color>("color") {
                Ok(col) => col.to_owned(),
                Err(_) => Color::default(),
            };
        } else {
            col = Color::default()
        }

        Box::new(ColorMaterial {
            color: col,
            texture: Some(tex.clone())
        })
    }

    fn key(&self) -> &str {
        COLOR_MATERIAL_LOADER_KEY
    }
}

pub trait AddMaterialLoader {
    fn add_prefab_material_loader<L: PrefabMaterialLoader, T: Asset + AssetDynamic>(&mut self, 

    ) -> &mut Self;
}

impl AddMaterialLoader for AppBuilder {
    
    /// Add a material loader for a certain material. This material will loaded into the entity 
    /// after the prefab is spawned. 
    fn add_prefab_material_loader<L: PrefabMaterialLoader, T: Asset + AssetDynamic>(
        &mut self,
    ) -> &mut Self {
        self.add_system(load_prefab_material::<L, T>.system())
    }
}
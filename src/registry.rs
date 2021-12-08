use crate::parse::{parse_prefab, LoadPrefabError, ReflectType};
use crate::prefab::Prefab;
use crate::processor::PrefabProcessor;

use bevy::prelude::{AppBuilder, ReflectComponent};
use bevy::reflect::TypeRegistration;
use std::fs;

use bevy::{
    reflect::{GetTypeRegistration, Reflect},
    utils::HashMap,
};

/// Manages and caches various types of prefab data.
#[derive(Default)]
pub struct PrefabRegistry {
    prefab_map: HashMap<String, Prefab>,
    type_info_map: HashMap<String, TypeInfo>,
    processor_map: HashMap<String, Box<dyn PrefabProcessor + Send + Sync>>,
}

impl PrefabRegistry {
    /// Register a new type that can be built from a `.prefab` file.
    pub fn register_type<T: Reflect + Default + GetTypeRegistration>(&mut self) -> &Self {
        let instance = T::default();
        let registration = T::get_type_registration();

        let info = TypeInfo {
            type_name: registration.short_name().to_string(),
            reflect_type: instance.reflect_ref().into(),
            registration,
        };

        self.type_info_map.insert(info.type_name.to_string(), info);
        self
    }


    /// Initialize a [PrefabProcessor] by type.
    pub fn init_processor<T: PrefabProcessor + Default + Send + Sync + 'static>(&mut self) {
        let p = T::default();
        self.processor_map.insert(p.key().to_string(), Box::new(p));
    }

    /// Add a [PrefabProcessor] to the registry.
    pub fn add_processor(&mut self, processor: Box<dyn PrefabProcessor + Send + Sync + 'static>) {
        self.processor_map
            .insert(processor.key().to_string(), processor);
    }
    
    /// Attempts to load a prefab from disk. 
    /// 
    /// Prefabs are loaded from the `assets/prefabs` directory. 
    /// If the prefab fails to load, it will return an error. If the prefab has already 
    /// been loaded, it will do nothing. Note this will cache the parsed prefab data so 
    /// it will stay in memory even if it's not longer used. You can use the `remove_prefab` 
    /// function  to remove it from the registry.
    pub fn load(&mut self, prefab_name: &str) -> Result<(),LoadPrefabError> {
        self.try_load(prefab_name).map(|_| ())
    }

    /// Remove a prefab from the registry. It will need to be loaded from disk
    /// again if re-loaded.
    pub fn remove_prefab(&mut self, prefab_name: &str) {
        self.prefab_map.remove(prefab_name);
    }

    /// Retrieve a prefab from the registry if it's been loaded yet.
    pub(crate) fn get_prefab(&self, prefab_name: &str) -> Option<&Prefab> {
        self.prefab_map.get(prefab_name)
    }

    /// Retrieve a registered [PrefabProcessor].
    pub(crate) fn get_processor(&self, key: &str) -> Option<&dyn PrefabProcessor> {
        match self.processor_map.get(key) {
            Some(processor) => Some(&**processor),
            None => None,
        }
    }

    /// Load the prefab from disk, or retrieve it if it's already been loaded.
    pub(crate) fn try_load(&mut self, prefab_name: &str) -> Result<&Prefab, LoadPrefabError> {
        if self.prefab_map.contains_key(prefab_name) {
            return Ok(&self.prefab_map[prefab_name]);
        }

        let path = ["assets/prefabs/", prefab_name].join("");
        let prefab_string = match fs::read_to_string(path) {
            Ok(str) => str,
            Err(e) => return Err(LoadPrefabError::PrefabFileReadError(e)),
        };
        match parse_prefab(&prefab_string, self) {
            Ok(prefab) => {
                let entry = self.prefab_map.entry(prefab_name.to_string());
                return Ok(entry.or_insert(prefab));
            }
            Err(e) => Err(e),
        }
    }


    pub(crate) fn reflect_component(&self, type_name: &str) -> Option<&ReflectComponent> {
        let registration = self.registration(type_name)?;
        registration.data::<ReflectComponent>()
    }

    pub(crate) fn type_info(&self, type_name: &str) -> Option<&TypeInfo> {
        self.type_info_map.get(type_name)
    }

    fn registration(&self, type_name: &str) -> Option<&TypeRegistration> {
        //self.type_map.get(type_name)
        match self.type_info_map.get(type_name) {
            Some(t) => Some(&t.registration),
            None => None,
        }
    }


}

pub trait PrefabRegisterType {
    fn register_prefab_type<T: Reflect + Default + GetTypeRegistration>(&mut self);
}

impl PrefabRegisterType for AppBuilder {
    fn register_prefab_type<T: Reflect + Default + GetTypeRegistration>(&mut self) {
        let world = self.world_mut();
        let mut reg = world.get_resource_mut::<PrefabRegistry>().unwrap();
        reg.register_type::<T>();
    }
}

pub trait PrefabRegisterProcessor {
    fn add_prefab_processor(&mut self, processor: Box<dyn PrefabProcessor + Send + Sync>);
    fn init_prefab_processor<T: PrefabProcessor + Default + Send + Sync + 'static>(&mut self);
}

impl PrefabRegisterProcessor for AppBuilder {
    fn init_prefab_processor<T: PrefabProcessor + Default + Send + Sync + 'static>(&mut self) {
        let t = T::default();
        self.add_prefab_processor(Box::new(t));
    }

    fn add_prefab_processor(&mut self, processor: Box<dyn PrefabProcessor + Send + Sync>) {
        let world = self.world_mut();
        let mut reg = world.get_resource_mut::<PrefabRegistry>().unwrap();
        reg.add_processor(processor);
    }
}

#[derive(Clone)]
pub(crate) struct TypeInfo {
    pub type_name: String,
    pub reflect_type: ReflectType,
    pub registration: TypeRegistration,
}

#[cfg(test)]
mod test {
    use super::PrefabRegistry;
    use crate::dynamic_cast::*;
    use bevy::reflect::{DynamicStruct, Reflect};

    #[derive(Reflect, Default)]
    struct TestComponentA;
    #[derive(Reflect, Default)]
    struct TestComponentB {
        x: i32,
    }

    #[test]
    fn load_test() {
        let mut reg = PrefabRegistry::default();
        reg.register_type::<TestComponentA>();
        reg.register_type::<TestComponentB>();

        let prefab = reg.try_load("test.prefab").unwrap();

        let components = prefab.components();

        assert_eq!(components.len(), 2);
        assert_eq!(components[0].name(), "TestComponentA");
        let compb = components[1].root();
        let compb = compb.cast_ref::<DynamicStruct>();

        assert_eq!(35, *compb.get::<i32>("x"));
    }
}
use crate::bundle::PrefabBundleLoader;
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

#[derive(Default)]
pub struct PrefabRegistry {
    prefab_map: HashMap<String, Prefab>,
    type_info_map: HashMap<String, TypeInfo>,
    //bundle_map: HashMap<String, Box<dyn FnMut(&mut EntityMut) + Send + Sync>>,
    bundle_map: HashMap<String, Box<dyn PrefabBundleLoader + Send + Sync>>,
    processor_map: HashMap<String, Box<dyn PrefabProcessor + Send + Sync>>,
}

impl PrefabRegistry {
    /// Register a new type of prefab.
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

    pub fn type_info(&self, type_name: &str) -> Option<&TypeInfo> {
        //println!("TYPENAME {}", type_name);
        self.type_info_map.get(type_name)
    }

    pub fn get_prefab(&self, prefab_name: &str) -> Option<&Prefab> {
        self.prefab_map.get(prefab_name)
    }

    pub fn load(&mut self, prefab_name: &str) -> Result<&Prefab, LoadPrefabError> {
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

    pub fn reflect_component(&self, type_name: &str) -> Option<&ReflectComponent> {
        let registration = self.registration(type_name)?;
        registration.data::<ReflectComponent>()
    }

    fn registration(&self, type_name: &str) -> Option<&TypeRegistration> {
        //self.type_map.get(type_name)
        match self.type_info_map.get(type_name) {
            Some(t) => Some(&t.registration),
            None => None,
        }
    }

    pub fn add_bundle_loader(&mut self, loader: Box<dyn PrefabBundleLoader + Send + Sync>) {
        self.bundle_map.insert(loader.key().to_string(), loader);
    }

    pub fn add_bundle_loader_t<T: PrefabBundleLoader + Default + Send + Sync + 'static>(&mut self) {
        let t = T::default();
        self.add_bundle_loader(Box::new(t));
    }

    pub fn get_bundle_loader(&self, name: &str) -> Option<&dyn PrefabBundleLoader> {
        if let Some(loader) = self.bundle_map.get(name) {
            return Some(&**loader);
        }
        None
    }

    pub fn get_processor(&self, key: &str) -> Option<&dyn PrefabProcessor> {
        match self.processor_map.get(key) {
            Some(processor) => Some(&**processor),
            None => None
        }
    }

    fn register_processor<T: PrefabProcessor + Default + Send + Sync + 'static>(&mut self) {
        let p = T::default();
        self.processor_map.insert(p.key().to_string(), Box::new(p));
    }

    fn add_processor(&mut self, processor: Box<dyn PrefabProcessor + Send + Sync + 'static>) {
        self.processor_map
            .insert(processor.key().to_string(), processor);
    }
}

pub trait PrefabRegisterType {
    fn prefab_register_type<T: Reflect + Default + GetTypeRegistration>(&mut self);
}

impl PrefabRegisterType for AppBuilder {
    fn prefab_register_type<T: Reflect + Default + GetTypeRegistration>(&mut self) {
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
        processor.on_init(self);
        let world = self.world_mut();
        let mut reg = world.get_resource_mut::<PrefabRegistry>().unwrap();
        reg.add_processor(processor);
    }
}

#[derive(Clone)]
pub struct TypeInfo {
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

        let prefab = reg.load("test.prefab").unwrap();

        let components = prefab.components();

        assert_eq!(components.len(), 2);
        assert_eq!(components[0].name(), "TestComponentA");
        let compb = components[1].root();
        let compb = compb.cast_ref::<DynamicStruct>();

        assert_eq!(35, *compb.get::<i32>("x"));
    }
}

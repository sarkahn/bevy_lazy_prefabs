use std::collections::hash_map::Entry;
use std::fs;
use crate::dynamic_cast::*;
use crate::parse::{parse, Rule, PrefabParserError};
use crate::prefab::Prefab;
use pest::{Parser, error::Error, iterators::Pair};

use bevy::{
    utils::HashMap,
    reflect::{
        Reflect, 
        GetTypeRegistration,
    },
};

#[derive(Default)]
pub struct PrefabRegistry {
    instance_map: HashMap<String, Box<dyn Reflect>>,
    prefab_map: HashMap<String, Prefab>,
}

pub enum PrefabRegistryError {
    UnregisteredComponent,
    
}

impl PrefabRegistry {
    /// Retreives the default instance for the given type name, or none if the type was never registered.
    pub fn instance(&self, type_name: &str) -> Option<&dyn Reflect> {
        match self.instance_map.get(type_name) {
            Some(instance) => Some(instance.as_ref()),
            None => None,
        }
    }

    /// Returns a reflected copy of the given instance, or none if the type was never registered.
    pub fn instance_clone(&self, type_name: &str) -> Option<Box<dyn Reflect>> {
        match self.instance(type_name) {
            Some(instance) => Some(instance.clone_value()),
            None => todo!(),
        }
    }

    /// Register a new type of prefab.
    pub fn register_component<T: Reflect + Default + GetTypeRegistration>(&mut self) 
    {
        let instance = T::default();
        let registration = T::get_type_registration();
        
        self.instance_map.insert(registration.short_name().to_string(), Box::new(instance));
    }

    pub fn load(&mut self, prefab_name: &str) -> Result<&Prefab, PrefabParserError> {
        if self.prefab_map.contains_key(prefab_name) {
            return Ok(&self.prefab_map[prefab_name]);
        }
        

        let path = ["assets/prefabs/", prefab_name].join("");
        let prefab_string = fs::read_to_string(path).expect("Error reading prefab");
        match parse(&prefab_string, self) {
            Ok(prefab) => {
                let entry = self.prefab_map.entry(prefab_name.to_string());
                return Ok(entry.or_insert(prefab)); 
            },
            Err(e) => return Err(e),
        };
    }
}

#[cfg(test)]
mod test {
    use bevy::reflect::{Reflect, DynamicStruct};
    use super::PrefabRegistry;
    use crate::dynamic_cast::*;

    #[derive(Reflect, Default)]
    struct TestComponentA;
    #[derive(Reflect, Default)]
    struct TestComponentB {
        x: i32,
    }

    #[test]
    fn load_test() {
        let mut reg = PrefabRegistry::default();
        reg.register_component::<TestComponentA>();
        reg.register_component::<TestComponentB>();

        let prefab = reg.load("test.prefab").unwrap();
        let components = &prefab.components;

        assert_eq!(prefab.components[0].name, "TestComponentA");
        let compb = &prefab.components[1].reflect;
        let compb = compb.cast_ref::<DynamicStruct>();
        assert_eq!(&15, compb.get::<i32>("x"));
    }

    #[derive(Reflect, Debug, Clone)]
    struct SomeComponent {
        i: i32,
        q: i32,
    }
    
    impl Default for SomeComponent {
        fn default() -> Self {
            Self { i: 0, q: 99 }
        }
    }
    
    #[test]
    fn clone_test() {
        let mut reg = PrefabRegistry::default();
    
        reg.register_component::<SomeComponent>();
    
        let mut b = reg.instance_clone("SomeComponent").unwrap();
    
        let mut prefab = DynamicStruct::default();
        prefab.insert("i", 15);
    
        b.apply(&prefab);
    
        let b = b.cast_mut::<DynamicStruct>();
    
        let bi = b.get::<i32>("i");
        let bq = b.get::<i32>("q");
        
        assert_eq!(bi, &15);
        assert_eq!(bq, &99);
    }
}
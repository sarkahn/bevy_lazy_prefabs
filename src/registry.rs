use crate::parse::{parse_prefab, LoadPrefabError};
use crate::prefab::Prefab;
use bevy::prelude::ReflectComponent;
use bevy::reflect::{ReflectRef, TypeRegistration};
use std::fs;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use bevy::{
    reflect::{GetTypeRegistration, Reflect},
    utils::HashMap,
};

#[derive(Default, Clone)]
pub struct PrefabRegistryArc {
    internal: Arc<RwLock<PrefabRegistry>>,
}

impl PrefabRegistryArc {
    pub fn read(&self) -> RwLockReadGuard<'_, PrefabRegistry> {
        self.internal.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, PrefabRegistry> {
        self.internal.write().unwrap()
    }

    pub fn register_type<T: Reflect + Default + GetTypeRegistration>(&mut self) {
        self.internal.write().unwrap().register_type::<T>();
    }
}

#[derive(Default)]
pub struct PrefabRegistry {
    prefab_map: HashMap<String, Prefab>,
    type_info_map: HashMap<String, TypeInfo>,
}

impl PrefabRegistry {
    /// Register a new type of prefab.
    pub fn register_type<T: Reflect + Default + GetTypeRegistration>(&mut self) {
        let instance = T::default();
        let registration = T::get_type_registration();

        let info = TypeInfo {
            type_name: registration.short_name().to_string(),
            reflect_type: instance.reflect_ref().into(),
            registration,
        };

        self.type_info_map.insert(info.type_name.to_string(), info);
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
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ReflectType {
    Struct,
    TupleStruct,
    Tuple,
    List,
    Map,
    Value,
}

impl<'a> From<ReflectRef<'a>> for ReflectType {
    fn from(reflect: ReflectRef) -> Self {
        match reflect {
            ReflectRef::Struct(_) => ReflectType::Struct,
            ReflectRef::TupleStruct(_) => ReflectType::TupleStruct,
            ReflectRef::Tuple(_) => ReflectType::Tuple,
            ReflectRef::List(_) => ReflectType::List,
            ReflectRef::Map(_) => ReflectType::Map,
            ReflectRef::Value(_) => ReflectType::Value,
        }
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
    use bevy::{
        reflect::{DynamicStruct, Reflect}, prelude::*, utils::HashMap,
    };

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
        let mut a = Handle::<Mesh>::default();
        a.init();
    }

    struct Map {
        map: HashMap<String, Box<dyn FnMut(&mut Box<dyn Reflect>)>>,
    }

    impl Map {
        fn insert(&mut self) {
            let func = |mesh: &mut Box<dyn Reflect>| {
                let mesh = mesh.downcast_ref::<Handle<Mesh>>().unwrap();
            };
            let func = Box::new(func);
            self.map.insert("Hi".to_string(), func);
        }
    }

    #[test]
    fn box_func() {
        let mut map = HashMap::default();

        map.insert("Hello", |mesh: &Box<dyn Reflect>| {
            let mesh = mesh.downcast_ref::<Handle<Mesh>>();
        });

    }
    trait ComponentInitializer {
        fn init(&mut self);
    }

    impl ComponentInitializer for Handle<Mesh> {
        fn init(&mut self) {
            
        }
    }
}

use crate::bundle::PrefabBundleLoader;
use crate::parse::{parse_prefab, LoadPrefabError};
use crate::prefab::{Prefab};
use bevy::ecs::world::EntityMut;
use bevy::prelude::{ReflectComponent, Bundle, AppBuilder};
use bevy::reflect::{ReflectRef, TypeRegistration, TypeRegistryInternal};
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

    pub fn add_bundle_loader(&mut self, loader: Box<dyn PrefabBundleLoader + Send + Sync>) {
        self.bundle_map.insert(loader.key().to_string(), loader);
    }

    pub fn get_bundle_loader(&self, name: &str) -> &dyn PrefabBundleLoader {
        &**self.bundle_map.get(name).unwrap()
    }


    // pub fn add_bundle_loader<T: Bundle>(&mut self, 
    //     name: &str,
    //     init_func: fn() -> T  
    // ) {
    //     let func = move |e: &mut EntityMut| {
    //         e.insert_bundle(init_func());
    //     };

    //     self.bundle_map.insert(name.to_string(), Box::new(func));
    // }

    // pub fn get_bundle_loader(&self, name: &str) -> 
    // Option<
    //     &mut Box<dyn FnMut(&mut EntityMut) + Send + Sync + 'static>
    // > {
    //     self.bundle_map.get_mut(name)
    // }
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
        reflect::{DynamicStruct, Reflect},
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
    }

}

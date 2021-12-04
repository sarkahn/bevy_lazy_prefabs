
use std::fs;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::dynamic_cast::*;
use crate::parse::{parse_prefab, LoadPrefabError};
use crate::prefab::Prefab;
use bevy::prelude::ReflectComponent;
use bevy::reflect::{TypeRegistration, ReflectRef};


use bevy::{
    prelude::*,
    utils::HashMap,
    reflect::{
        Reflect, 
        GetTypeRegistration,
    },
};

#[derive(Default, Clone)]
pub struct PrefabRegistryArc {
    pub internal: Arc<RwLock<PrefabRegistry>>,
}

impl PrefabRegistryArc {
    pub fn read(&self) -> RwLockReadGuard<'_, PrefabRegistry> {
        self.internal.read().unwrap()
    }

    pub fn write(&self) ->  RwLockWriteGuard<'_, PrefabRegistry> {
        self.internal.write().unwrap()
    }
}

#[derive(Default)]
pub struct PrefabRegistry {
    //instance_map: HashMap<String, Box<dyn Reflect>>,
    prefab_map: HashMap<String, Prefab>,
    //type_map: HashMap<String, TypeRegistration>,
    //reflect_type_map: HashMap<String, ReflectType>,
    type_info_map: HashMap<String, TypeInfo>,
}

// impl Clone for PrefabRegistry {
//     fn clone(&self) -> Self {
//         Self { 
//             prefab_map: self.prefab_map.clone(), 
//             type_info_map: self.type_info_map.clone() 
//         }
//     }
// }

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

impl TypeInfo {
    pub fn reflect_component(&self) -> Option<&ReflectComponent> {
        self.registration.data::<ReflectComponent>()
    }

    pub fn add_component<T: Reflect + 'static>(&self, world: &mut World, entity: Entity, component: T) {
        let rc = self.registration.data::<ReflectComponent>().unwrap();
        rc.add_component(world, entity, &component);
    }
}

impl PrefabRegistry {
    /// Retreives the default instance for the given type name, or none if the type was never registered.
    // pub fn instance(&self, type_name: &str) -> Option<&dyn Reflect> {
    //     match self.instance_map.get(type_name) {
    //         Some(instance) => Some(instance.as_ref()),
    //         None => None,
    //     }
    // }

    /// Returns a reflected copy of the given instance, or none if the type was never registered.
    // pub fn instance_clone(&self, type_name: &str) -> Option<Box<dyn Reflect>> {
    //     match self.instance(type_name) {
    //         Some(instance) => Some(instance.clone_value()),
    //         None => todo!(),
    //     }
    // }

    /// Register a new type of prefab.
    pub fn register_component<T: Reflect + Default + GetTypeRegistration>(&mut self) 
    {
        let instance = T::default();
        let registration = T::get_type_registration();

        let info = TypeInfo {
            type_name: registration.short_name().to_string(),
            reflect_type: instance.reflect_ref().into(),
            registration
        };

        self.type_info_map.insert(info.type_name.to_string(), info);
        //self.reflect_type_map.insert(registration.short_name().to_string(), instance.reflect_ref().into());
        
        //self.instance_map.insert(registration.short_name().to_string(), Box::new(instance));
        //self.type_map.insert(registration.short_name().to_string(), registration);
    }

    pub fn registration(&self, type_name: &str) -> Option<&TypeRegistration> {
        //self.type_map.get(type_name)
        match self.type_info_map.get(type_name) {
            Some(t) => Some(&t.registration),
            None => None
        }
    }

    pub fn reflect_component(&self, type_name: &str) -> Option<&ReflectComponent> {
        let registration = self.registration(type_name)?;
        registration.data::<ReflectComponent>()
    }

    pub fn reflect_clone(&self, type_name: &str) -> ReflectComponent {
        let reg = self.registration(type_name).unwrap().clone();
        reg.data::<ReflectComponent>().unwrap().to_owned()
    }

    pub fn load(&mut self, prefab_name: &str) -> Result<&Prefab, LoadPrefabError> {
        if self.prefab_map.contains_key(prefab_name) {
            return Ok(&self.prefab_map[prefab_name]);
        }
        
        let path = ["assets/prefabs/", prefab_name].join("");
        let prefab_string = match fs::read_to_string(path) {
            Ok(str) => str,
            Err(e) => return Err(LoadPrefabError::FileReadError(e)),
        };
        match parse_prefab(&prefab_string, self) {
            Ok(prefab) => {
                let entry = self.prefab_map.entry(prefab_name.to_string());
                return Ok(entry.or_insert(prefab)); 
            },
            Err(e) => return Err(e),
        };
    }

    pub fn reflect_type(&self, type_name: &str) -> Option<&ReflectType> {
        match self.type_info_map.get(type_name) {
            Some(t) => Some(&t.reflect_type),
            None => None
        }
    }

    pub fn type_info(&self, type_name: &str) -> Option<&TypeInfo> {
        //println!("TYPENAME {}", type_name);
        self.type_info_map.get(type_name)
    }

    pub fn get_prefab(&self, prefab_name: &str) -> Option<&Prefab> {
        self.prefab_map.get(prefab_name)
    }

    pub fn register_bundle<T: Bundle>(&self) {
        let bundle_name = std::any::type_name::<T>();
        let bundle_name = TypeRegistration::get_short_name(bundle_name);
        println!("Components for {}", bundle_name);
        for t in T::type_info() {
            let component_name = TypeRegistration::get_short_name(t.type_name());
            println!("{}", component_name);
        }
    }
}

#[cfg(test)]
mod test {
    use bevy::{reflect::{Reflect, DynamicStruct, DynamicTupleStruct, DynamicTuple}, prelude::*};
    use super::PrefabRegistry;
    use crate::{dynamic_cast::*, registry::ReflectType};

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

        let components = prefab.components();

        assert_eq!(components.len(), 2);
        assert_eq!(components[0].name(), "TestComponentA");
        let compb = components[1].root();
        let compb = compb.cast_ref::<DynamicStruct>();

        assert_eq!(35, *compb.get::<i32>("x"));
    }
    
    #[test]
    fn bundle_test() {
        let mut reg = PrefabRegistry::default();
        reg.register_bundle::<PbrBundle>();
    }


    #[test]
    fn vec_test() {
        let mut v = Vec3::default();
        let mut d = DynamicTuple::default();

        let v = v.clone_value();

    }
}
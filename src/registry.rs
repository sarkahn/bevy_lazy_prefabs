use std::{sync::Arc, fs};

use bevy::{
    prelude::*,
    reflect::{GetTypeRegistration, ReflectRef, TypeRegistration},
    utils::HashMap,
};

use crate::{
    commands::PrefabCommand, 
    prefab::Prefab, 
    parse::LoadPrefabError,
    parse::parse_prefab_string,
};

#[derive(Default)]
pub struct PrefabRegistry {
    type_data: HashMap<String, TypeInfo>,
    commands: HashMap<String, Arc<dyn PrefabCommand + Send + Sync + 'static>>,
    prefabs: HashMap<String, Arc<Prefab>>,
}

impl PrefabRegistry {
    pub fn register_type<T: Reflect + GetTypeRegistration + Default>(&mut self) {
        let reg = T::get_type_registration();
        let instance = T::default();
        let name = reg.short_name().to_string();

        let info = TypeInfo {
            type_name: name.clone(),
            reflect_type: instance.reflect_ref().into(),
            registration: reg,
        };

        self.type_data.insert(name, info);
    }

    pub(crate) fn get_type_data(&self, name: &str) -> Option<&TypeInfo> {
        self.type_data.get(name)
    }

    pub fn register_command<T: PrefabCommand + Default + Send + Sync + 'static>(
        &mut self,
    ) {
        let t = T::default();
        self.commands.insert(t.key().to_string(), Arc::new(t));
    }

    pub fn get_command(
        &self,
        name: &str,
    ) -> Option<&Arc<dyn PrefabCommand + Send + Sync + 'static>> {
        self.commands.get(name)
    }

    /// Load the prefab from disk, or retrieve it if it's already been loaded.
    pub fn load(&mut self, name: &str) -> Result<&Arc<Prefab>, LoadPrefabError> {
        if self.prefabs.contains_key(name) {
            return Ok(self.prefabs.get(name).unwrap())
        };

        let path = ["assets/prefabs/", name].join("");

        let prefab_string = match fs::read_to_string(path) {
            Ok(str) => str,
            Err(e) => return Err(LoadPrefabError::FileReadError(e)),
        };

        match parse_prefab_string(&prefab_string, self) {
            Ok(prefab) => {
                //let entry = self.prefab_map.entry(prefab_name.to_string());
                let entry = self.prefabs.entry(name.to_string());
                return Ok(entry.or_insert(Arc::new(prefab)));
            }
            Err(e) => return Err(e),
        };
    }
}

pub(crate) struct TypeInfo {
    #[allow(dead_code)]
    pub type_name: String,
    pub reflect_type: ReflectType,
    pub registration: TypeRegistration,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) enum ReflectType {
    Struct,
    TupleStruct,
    Tuple,
    List,
    Map,
    Value,
}

impl From<ReflectRef<'_>> for ReflectType {
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

#[cfg(test)]
mod test {
    
}

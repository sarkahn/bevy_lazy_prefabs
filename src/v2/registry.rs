use std::sync::Arc;

use bevy::{
    prelude::*, 
    utils::HashMap, 
    reflect::{TypeRegistration, GetTypeRegistration}};

use super::prefab_command::PrefabCommand;

pub struct PrefabRegistry {
    type_data: HashMap<String, TypeRegistration>,
    commands: HashMap<String, Arc<dyn PrefabCommand + Send + Sync + 'static>>,
}

impl PrefabRegistry {
    pub fn register_type<T: GetTypeRegistration>(&mut self, name: &str) {

        let info = T::get_type_registration();

        self.type_data.insert(name.to_string(), info);
    }

    pub fn get_type_data(&self, name: &str) -> Option<&TypeRegistration> {
        self.type_data.get(name)
    }

    pub fn register_command<T: PrefabCommand + Send + Sync + 'static>(&mut self, name: &str, command: T) {
        self.commands.insert(name.to_string(), Arc::new(command));
    }

    pub fn get_command(&self, name: &str) -> Option<&Arc<dyn PrefabCommand + Send + Sync + 'static>> {
        self.commands.get(name)
    }
}
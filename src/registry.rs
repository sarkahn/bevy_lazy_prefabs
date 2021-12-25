use std::sync::Arc;

use bevy::{
    prelude::*,
    reflect::{GetTypeRegistration, ReflectRef, TypeRegistration},
    utils::HashMap,
};

use crate::commands::PrefabCommand;

#[derive(Default)]
pub struct PrefabRegistry {
    type_data: HashMap<String, TypeInfo>,
    commands: HashMap<String, Arc<dyn PrefabCommand + Send + Sync + 'static>>,
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

    pub fn register_command<T: PrefabCommand + Send + Sync + 'static>(
        &mut self,
        name: &str,
        command: T,
    ) {
        self.commands.insert(name.to_string(), Arc::new(command));
    }

    pub fn get_command(
        &self,
        name: &str,
    ) -> Option<&Arc<dyn PrefabCommand + Send + Sync + 'static>> {
        self.commands.get(name)
    }
}

pub(crate) struct TypeInfo {
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
    use super::PrefabRegistry;
    use crate::{commands::PrefabCommand, dynamic_cast::*};
    use bevy::{
        prelude::*,
        reflect::{DynamicStruct, Reflect},
    };

    #[derive(Reflect, Default)]
    #[reflect(Component)]
    struct TestComponentA;

    #[derive(Reflect, Default)]
    #[reflect(Component)]
    struct TestComponentB {
        x: i32,
    }

    // #[test]
    // fn load_test() {
    //     let mut reg = PrefabRegistry::default();
    //     reg.register_type::<TestComponentA>();
    //     reg.register_type::<TestComponentB>();

    //     let prefab = reg.try_load("test.prefab").unwrap();

    //     let commands = prefab.commands();

    //     assert_eq!(commands.len(), 2);

    //     let component = match &commands[0] {
    //         PrefabCommand::AddComponent(comp) => comp,
    //         _ => unreachable!(),
    //     };

    //     assert_eq!(component.name(), "TestComponentA");

    //     let compb = match &commands[1] {
    //         PrefabCommand::AddComponent(comp) => comp,
    //         _ => unreachable!(),
    //     };
    //     let root = compb.root();

    //     let root = root.cast_ref::<DynamicStruct>();

    //     assert_eq!(35, *root.get::<i32>("x"));
    // }
}

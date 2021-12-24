use crate::PrefabLoader;
use crate::parse::{parse_prefab_string, LoadPrefabError, ReflectType};
use crate::prefab::Prefab;
use crate::processor::PrefabProcessor;

use bevy::prelude::AppBuilder;
use bevy::reflect::TypeRegistration;
use std::fs;
use std::sync::Arc;

use bevy::{
    reflect::{GetTypeRegistration, Reflect},
    utils::HashMap,
};

/// Manages and caches prefab related data.
#[derive(Default)]
pub struct PrefabRegistry {
    type_info_map: HashMap<String, TypeInfo>,
    processors: HashMap<String, Arc<dyn PrefabProcessor + Send + Sync>>,

    prefab_map: HashMap<String, Arc<Prefab>>,
    loaders: HashMap<String, Arc<dyn PrefabLoader + Send + Sync>>,
}

impl PrefabRegistry {
    /// Register a new type that can be built from a *.prefab* file.
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
        self.add_processor(Arc::new(p));
    }

    /// Add a [PrefabProcessor] to the registry.
    pub fn add_processor(&mut self, processor: Arc<dyn PrefabProcessor + Send + Sync + 'static>) {
        self.processors
            .insert(processor.key().to_string(), processor);
    }

    /// Attempts to load a prefab from disk.
    ///
    /// Prefabs are loaded from the *assets/prefabs* directory.
    /// If the prefab fails to load, it will return an error. If the prefab has already
    /// been loaded, it will do nothing. 
    /// 
    /// Note this will cache the parsed prefab data so it will stay in memory even if it's 
    /// no longer used. You can use the `remove` function  to remove it from the registry.
    pub fn load(&mut self, prefab_name: &str) -> Result<(), LoadPrefabError> {
        self.try_load(prefab_name).map(|_| ())
    }
    
    /// Remove the prefab from the registry cache.
    pub fn remove(&mut self, prefab_name: &str) {
        self.prefab_map.remove(prefab_name);
    }

    /// Load the prefab from disk, or retrieve it if it's already been loaded.
    pub(crate) fn try_load(
        &mut self, 
        prefab_name: &str
    ) -> Result<&Arc<Prefab>, LoadPrefabError> {
        // https://rust-lang.github.io/rfcs/2094-nll.html#problem-case-3-conditional-control-flow-across-functions
        if self.prefab_map.contains_key(prefab_name) {
            return Ok(self.prefab_map.get(prefab_name).unwrap())
        }
    
        let path = ["assets/prefabs/", prefab_name].join("");
        let prefab_string = match fs::read_to_string(path) {
            Ok(str) => str,
            Err(e) => return Err(LoadPrefabError::PrefabFileReadError(e)),
        };
        match parse_prefab_string(&prefab_string, self) {
            Ok(prefab) => {
                //let entry = self.prefab_map.entry(prefab_name.to_string());
                let entry = self.prefab_map.entry(prefab_name.to_string());
                return Ok(entry.or_insert(Arc::new(prefab)));
            }
            Err(e) => return Err(e),
        };
    }

    pub(crate) fn type_info(&self, type_name: &str) -> Option<&TypeInfo> {
        self.type_info_map.get(type_name)
    }

    pub(crate) fn get_processor(
        &self,
        key: &str,
    ) -> Option<&Arc<dyn PrefabProcessor + Send + Sync + 'static>> {
        self.processors.get(key)
    }

    pub(crate) fn get_prefab(&self, name: &str) -> Option<&Arc<Prefab>> {
        self.prefab_map.get(name)
    }

    pub(crate) fn get_loader(&self, name: &str) -> Option<&Arc<dyn PrefabLoader + Send + Sync + 'static>> {
        self.loaders.get(name)
    }
}

/// An [AppBuilder] trait for registering prefab components.
pub trait PrefabRegisterType {
    /// Register a new component type that can be built from a *.prefab* file. 
    /// 
    /// Custom prefab components must include the `#[derive(Reflect, Default)]` and 
    /// `#[reflect(Component)]` attributes in order to be built properly from a prefab file.
    /// 
    /// # Example 
    /// 
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_lazy_prefabs::*;
    /// 
    /// #[derive(Reflect, Default)]
    /// #[reflect(Component)]
    /// struct SomeComponent {
    ///     v: i32,
    /// }
    /// 
    /// fn main () {
    ///     App::build()
    ///     .add_plugins(DefaultPlugins)
    ///     .add_plugin(LazyPrefabsPlugin)
    ///     .register_prefab_type::<SomeComponent>()
    ///     .run();
    /// }
    /// ```
    fn register_prefab_type<T: Reflect + Default + GetTypeRegistration>(&mut self) -> &mut Self;
}

impl PrefabRegisterType for AppBuilder {
    fn register_prefab_type<T: Reflect + Default + GetTypeRegistration>(&mut self) -> &mut Self {
        let world = self.world_mut();
        let mut reg = world.get_resource_mut::<PrefabRegistry>().unwrap();
        reg.register_type::<T>();
        self
    }
}

pub trait PrefabRegisterProcessor {
    fn add_prefab_processor(&mut self, processor: Arc<dyn PrefabProcessor + Send + Sync>);
    fn init_prefab_processor<T: PrefabProcessor + Default + Send + Sync + 'static>(&mut self);
}

impl PrefabRegisterProcessor for AppBuilder {
    fn init_prefab_processor<T: PrefabProcessor + Default + Send + Sync + 'static>(&mut self) {
        let t = T::default();
        self.add_prefab_processor(Arc::new(t));
    }

    fn add_prefab_processor(
        &mut self,
        processor: Arc<dyn PrefabProcessor + Send + Sync + 'static>,
    ) {
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

    #[test]
    fn load_test() {
        let mut reg = PrefabRegistry::default();
        reg.register_type::<TestComponentA>();
        reg.register_type::<TestComponentB>();

        let prefab = reg.try_load("test.prefab").unwrap();

        let commands = prefab.commands();

        assert_eq!(commands.len(), 2);

        let component = match &commands[0] {
            PrefabCommand::AddComponent(comp) => comp,
            _ => unreachable!(),
        };

        assert_eq!(component.name(), "TestComponentA");

        let compb = match &commands[1] {
            PrefabCommand::AddComponent(comp) => comp,
            _ => unreachable!(),
        };
        let root = compb.root();

        let root = root.cast_ref::<DynamicStruct>();

        assert_eq!(35, *root.get::<i32>("x"));
    }
}

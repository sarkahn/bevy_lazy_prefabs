use std::sync::Arc;

use bevy::{ecs::system::{Command, EntityCommands}, prelude::*};

use crate::{
    prefab::{Prefab, PrefabComponent, PrefabLoad, PrefabProcessorData},
    registry::PrefabRegistry, parse::LoadPrefabError
};

// /// Adds the `spawn_prefab` option to bevy [Commands].
// pub trait SpawnPrefabCommands<'w> {
//     /// Spawn a prefab from a ".prefab" file.
//     /// 
//     /// The prefab name should include the file extension. Prefabs files are loaded
//     /// from the *assets/prefabs* directory.
//     /// 
//     /// # Example: 
//     /// 
//     /// ```
//     /// use bevy::prelude::*;
//     /// use bevy_lazy_prefabs::*;
//     /// 
//     /// fn setup(mut commands: Commands) {
//     ///   commands.spawn_prefab("sprite.prefab");
//     ///   commands.spawn_prefab("camera.prefab");
//     ///  }
//     /// ```
//     fn spawn_prefab<'a>(&'a mut self, prefab_name: &str) -> EntityCommands<'w, 'a>;
// }

// struct SpawnPrefab {
//     prefab_name: String,
// }

// impl Command for SpawnPrefab {
//     fn write(self: Box<Self>, world: &mut World) {
//         world.resource_scope(|world, mut reg: Mut<PrefabRegistry>| {
//             reg.load(self.prefab_name.as_str()).unwrap();

//             let prefab = reg
//                 .get_prefab(self.prefab_name.as_str())
//                 .unwrap()
//                 .clone();

//             let entity = world.spawn().id();
//             load_prefab(prefab, &mut reg, world, entity);
//         });
//     }
// }

// impl<'w> SpawnPrefabCommands<'w> for Commands<'w> {
//     fn spawn_prefab<'a>(&'a mut self, prefab_name: & str) -> EntityCommands<'w, 'a> {
//         self.add(SpawnPrefab {
//             prefab_name: prefab_name.to_string(),
//         });
//         todo!()
//     }
// }

// pub struct SpawnPrefabFromString {
//     prefab_name: String,
//     input: String,
// }
// impl Command for SpawnPrefabFromString {
//     fn write(self: Box<Self>, world: &mut World) {
//         world.resource_scope(|world, mut reg: Mut<PrefabRegistry>| {
//             let prefab = reg.load_from_string(self.prefab_name.as_ref(), self.input.as_ref()).unwrap();

//             let entity = world.spawn().id();
//             load_prefab(prefab.clone(), &mut reg, world, entity);
//         });
//     }
// }

pub trait SpawnPrefabCommands<'a> {
    fn spawn_prefab<'b>( &'b mut self, 
        Prefab: Handle<Prefab>, 
    ) -> EntityCommands<'a, 'b>;
}
impl<'a> SpawnPrefabCommands<'a> for Commands<'a> {
    fn spawn_prefab<'b>(
        &'b mut self, 
        prefab: Handle<Prefab>,
    ) -> EntityCommands<'a, 'b> {
        let mut entity = self.spawn();
        let id = entity.id();

        entity.insert(LoadPrefabCommand {
            entity: id,
            handle: prefab,
        });
        entity
    }
}

struct LoadPrefabCommand {
    entity: Entity,
    handle: Handle<Prefab>,
}

impl Command for LoadPrefabCommand {
    fn write(self: Box<Self>, world: &mut World) {
        let entity = self.entity;
        let handle = self.handle;
        world.resource_scope(|world, mut prefabs: Mut<Assets<Prefab>>| {
            if let Some(prefab) =  prefabs.get(handle) {
                //let prefab = Arc::new(prefab);
                //load_prefab_aahh(prefab.clone(), world, entity);
            }
        });
    }
}

fn load_prefab_command(prefab: &Prefab, registry: &mut PrefabRegistry, entity: Entity, world: &mut World) {

}



pub(crate) enum PrefabCommand {
    AddComponent(Arc<PrefabComponent>),
    Processor(Arc<PrefabProcessorData>),
    LoadPrefab(Arc<PrefabLoad>),
}

pub trait SpawnPrefabCommands2<'a> {
    fn spawn_prefab<'b>( &'b mut self, 
        prefab_name: &str, 
        registry: &mut PrefabRegistry
    ) -> Result<EntityCommands<'a, 'b>, LoadPrefabError>;
}
impl<'a> SpawnPrefabCommands2<'a> for Commands<'a> {
    fn spawn_prefab<'b>(
        &'b mut self, 
        prefab_name: & str,
        registry: &mut PrefabRegistry
    ) -> Result<EntityCommands<'a, 'b>, LoadPrefabError> {

        let prefab = registry.try_load(prefab_name)?.clone();

        let id = self.spawn().id();

        for command in prefab.commands() {
            match command {
                PrefabCommand::AddComponent(comp) => {
                    self.add(PrefabAddComponent {
                        entity: id,
                        comp: comp.clone(),
                    });
                }
                PrefabCommand::Processor(proc) => {
                    self.add(PrefabProcessor {
                        entity: id,
                        data: proc.clone(),
                    });
                }
                PrefabCommand::LoadPrefab(load) => {
                    self.add(PrefabLoadCommand {
                        entity: id, 
                        load: load.clone()
                    });
                }
            }
        }

        Ok(self.entity(id))
    }
}

fn parse_prefab_system(
    server:  Res<AssetServer>,
    mut registry: ResMut<PrefabRegistry>,
    mut prefab_events: EventReader<AssetEvent<Prefab>>,
) {
    for event in prefab_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                
            },
            AssetEvent::Modified { handle } => {
                
            },
            AssetEvent::Removed { handle } => {
                
            },
        }
    }
}


struct PrefabLoadCommand {
    entity: Entity,
    load: Arc<PrefabLoad>,
}
impl Command for PrefabLoadCommand {
    fn write(self: Box<Self>, world: &mut World) {
        world.resource_scope(|world, mut reg: Mut<PrefabRegistry>| {
            reg.load(self.load.path()).unwrap();

            let prefab = reg
                .get_prefab(self.load.path())
                .unwrap()
                .clone();

            load_prefab(prefab, &mut reg, world, self.entity);
        });
    }
}

fn load_prefab_aahh(prefab: &Prefab, entity: Entity, world: &mut World) {
    for command in prefab.commands() {
        match command {
            PrefabCommand::AddComponent(comp) => {
                add_component(comp.clone(), entity, world);
            }
            PrefabCommand::Processor(proc) => {
                run_processor(proc.clone(), entity, world);
            }
            PrefabCommand::LoadPrefab(load) => {
                load_prefab_aahh(prefab.clone(), entity, world);
            }
        }
    }
}

fn add_component(comp: Arc<PrefabComponent>, entity: Entity, world: &mut World) {
    let type_id = comp.type_id();
    if world.entity(entity).contains_type_id(type_id) {
        comp.reflect().apply_component(world, entity, comp.root());
    } else {
        comp.reflect().add_component(world, entity, comp.root());
    }   
}

fn run_processor(data: Arc<PrefabProcessorData>, entity: Entity, world: &mut World) {
    
    data.processor()
    .process_prefab(data.properties(), world, entity);
}

struct PrefabAddComponent {
    entity: Entity,
    comp: Arc<PrefabComponent>,
}
impl Command for PrefabAddComponent {
    fn write(self: Box<Self>, world: &mut World) {
        let comp = &self.comp;
        let entity = self.entity;
        let type_id = comp.type_id();
        if world.entity(entity).contains_type_id(type_id) {
            comp.reflect().apply_component(world, entity, comp.root());
        } else {
            comp.reflect().add_component(world, entity, comp.root());
        }
    }
}

struct PrefabProcessor {
    entity: Entity,
    data: Arc<PrefabProcessorData>,
}
impl Command for PrefabProcessor {
    fn write(self: Box<Self>, world: &mut World) {
        self.data.processor()
        .process_prefab(self.data.properties(), world, self.entity);
    }
}

fn load_prefab(
    prefab: Arc<Prefab>,
    registry: &mut PrefabRegistry,
    world: &mut World,
    entity: Entity,
) {
    for command in prefab.commands() {
        process_command(&command, world, registry, entity);
    }
}

fn process_command(
    command: &PrefabCommand,
    world: &mut World,
    registry: &mut PrefabRegistry,
    entity: Entity,
) {
    match command {
        PrefabCommand::AddComponent(comp) => {
            let type_id = comp.type_id();
            if world.entity(entity).contains_type_id(type_id) {
                comp.reflect().apply_component(world, entity, comp.root());
            } else {
                comp.reflect().add_component(world, entity, comp.root());
            }
        }
        PrefabCommand::Processor(proc) => {
            proc.processor()
                .process_prefab(proc.properties(), world, entity);
        }
        PrefabCommand::LoadPrefab(load) => {
            registry.load(load.path()).unwrap();
            let prefab = registry.get_prefab(load.path()).unwrap().clone();

            load_prefab(prefab, registry, world, entity);
        }
    }
}
use bevy::{prelude::*, reflect::DynamicStruct};

pub trait PrefabLoader {
    fn run(&self, data: Option<DynamicStruct>, world: &mut World, entity: Entity);
    fn key(&self) -> &str;
    //fn parse() -> Self;
}

pub struct LoadColorMaterial(String);

impl PrefabLoader for LoadColorMaterial {
    fn run(&self, data: Option<DynamicStruct>, world: &mut World, entity: Entity) {
        println!("Loading color material yo");
    }

    fn key(&self) -> &str {
        "LoadColorMaterial"
    }
}
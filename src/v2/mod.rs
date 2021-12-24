use bevy::prelude::*;

mod prefab_command;
mod prefab_builder;
mod commands;
mod prefab;
mod parse;
mod registry;

pub struct V2Plugin;

impl Plugin for V2Plugin {
    fn build(&self, app: &mut AppBuilder) {
        
    }
}
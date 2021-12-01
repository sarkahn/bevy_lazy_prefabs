use bevy::prelude::*;

struct PrefabComponent {
    name: String,
    reflect: Box<dyn Reflect>,
}

struct PrefabReflect {
    name: String,
    components: Vec<Box<dyn Reflect>>,
}
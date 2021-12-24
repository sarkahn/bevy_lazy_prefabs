use bevy::reflect::TypeUuid;

#[derive(Debug, TypeUuid)]
#[uuid = "6ea14da5-6bf8-3ea1-9886-1d7bf6c17d2f"]
pub struct Prefab {
    data: String,
}
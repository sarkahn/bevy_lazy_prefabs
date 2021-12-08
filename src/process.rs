use bevy::prelude::*;

// use crate::prefab::{Prefab, PrefabAssetType};

// fn load_prefab_assets(
//     q_prefab: Query<&Prefab, Added<Prefab>>,
//     asset_server: Res<AssetServer>,
//     mut color_materials: ResMut<Assets<ColorMaterial>>,
//     mut standard_materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     for pfb in q_prefab.iter() {
//         if let Some(assets) = pfb.assets() {
//             for (name, asset) in assets.iter() {
//                 match asset.asset_type() {
//                     PrefabAssetType::ColorMaterial => {
//                         let tex = asset_server.load(asset.path());
//                         color_materials.add(tex.into());
//                     },
//                     PrefabAssetType::StandardMaterial => todo!(),
//                     PrefabAssetType::Mesh => todo!(),
//                 }
//             }
//         }
//     }
// }
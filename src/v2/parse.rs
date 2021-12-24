use bevy::{
    prelude::*,
    reflect::{DynamicList, DynamicStruct, DynamicTuple, DynamicTupleStruct, Reflect, ReflectRef},
};
use pest::{error::Error, iterators::Pair, Parser};
use pest_derive::*;
use std::{any::Any, ops::Range, sync::Arc};
use thiserror::Error;

use super::{prefab::Prefab, };

// #[derive(Parser)]
// #[grammar = "lazy_prefabs.pest"]
// struct PrefabParser;

// pub(crate) fn parse_prefab_string(input: &str) -> Result<Vec<PrefabBuilderCommand>, LoadPrefabError> {
//     let pair = match PrefabParser::parse(Rule::prefab, input) {
//         Ok(parsed) => parsed,
//         Err(e) => return Err(LoadPrefabError::PestParseError(e)),
//     }.next().unwrap();

//     let pair = pair.into_inner();

//     for field in pair {
//         match field.as_rule() {
//             // AddComponent
//             Rule::component => {
//                 todo!()
//             },
//             // PrefabCommand
//             Rule::processor => {
//                 todo!()
//             },
//             _ => unreachable!()
//         }
//     }

//     todo!()
// }



// #[derive(Error, Debug)]
// pub enum LoadPrefabError {
//     #[error("Pest error parsing prefab string.")]
//     PestParseError(#[from] Error<Rule>),
// }
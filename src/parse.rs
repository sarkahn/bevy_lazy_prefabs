
use std::ops::Range;

use thiserror::Error;

use bevy::reflect::{Reflect, DynamicStruct, DynamicList, TupleStruct, DynamicTupleStruct, DynamicTuple};
use pest::{Parser, error::Error, iterators::{Pair, Pairs}};
use pest_derive::*;

use crate::{prefab::*, registry::{PrefabRegistry, ReflectType, TypeInfo}};
use crate::dynamic_cast::*;

#[derive(Parser)]
#[grammar = "lazy_prefabs.pest"]
struct PrefabParser;

#[derive(Error, Debug)]
pub enum LoadPrefabError {
    #[error("Error reading prefab file")]
    PrefabFileReadError(#[from]std::io::Error),
    #[error("Error parsing component - {0} was not registered with the PrefabRegistry")]
    UnregisteredComponent(String),
    #[error("Error parsing prefab")]
    ParserError(#[from] Error<Rule>),
}


pub fn parse_prefab(input: &str, registry: &PrefabRegistry) -> Result<Prefab, LoadPrefabError> {

    let parsed = match PrefabParser::parse(Rule::prefab, input) {
        Ok(parsed) => parsed,
        Err(e) => return Err(LoadPrefabError::ParserError(e)),
    };

    let mut prefab_name = None;
    let mut components = Vec::new();

    for prefab_parse in parsed {
        match prefab_parse.as_rule() {
            Rule::prefab_name => {
                prefab_name = Some(prefab_parse.as_str().to_string());
            },
            // Type Name, Components
            Rule::component => {
                let mut component_parse = prefab_parse.into_inner();

                let type_name = component_parse.next().unwrap().as_str();

                let type_info = match registry.type_info(type_name) {
                    Some(t) => t,
                    None => return Err(LoadPrefabError::UnregisteredComponent(type_name.to_string())),
                };

                let fields = parse_fields(component_parse);

                let root = match fields {
                    Some(fields) => Some(build_root(type_info, fields)),
                    None => None,
                };

                let comp = match root {
                    Some(root) => PrefabComponent::new(type_name, root),
                    None => PrefabComponent::from_type(type_info),
                };

                components.push(comp);
            },
            _ => unreachable!()
        }
    }

    Ok(Prefab::new(prefab_name, components))
}

fn parse_fields(component_parse: Pairs<Rule>) -> Option<Vec<ReflectField>> {
    let mut fields = None;
                
    // Field Name, Field Value
    for field_parse in component_parse {
        let mut field_parse = field_parse.into_inner(); 
    
        let fields = fields.get_or_insert(Vec::new());

        let field_name = field_parse.next().unwrap().as_str();
        let value_parse = field_parse.next().unwrap();

        let field = ReflectField {
            name: field_name.to_string(),
            value: parse_value(value_parse),
        };
        
        fields.push(field);
    }
    fields
}

/// Build a root object from a list of fields
fn build_root(type_info: &TypeInfo, fields: Vec<ReflectField>) -> Box<dyn Reflect> {
    match type_info.reflect_type {
        ReflectType::Struct => {
            let mut root = DynamicStruct::default();
            for field in fields {
                root.insert_boxed(&field.name, field.value);
            }
            Box::new(root)
        },
        ReflectType::TupleStruct => {
            let mut root = DynamicTupleStruct::default();
            for field in fields {
                root.insert_boxed(field.value);
            }
            Box::new(root)
        },
        ReflectType::Tuple => {
            let mut root = DynamicTuple::default();
            for field in fields {
                root.insert_boxed(field.value);
            }
            Box::new(root)
        },
        ReflectType::List => todo!(),
        ReflectType::Map => todo!(),
        ReflectType::Value => todo!(),
    }
}

fn parse_value(pair: Pair<Rule>) -> Box<dyn Reflect> {
    let value_string = pair.as_str();
    match pair.as_rule() {
        Rule::int => {
            let num = value_string.parse::<i32>().expect(
                "Error parsing int FieldValue"
            );
            Box::new(num)
        },
        Rule::float => {
            let f = value_string.parse::<f32>().expect(
                "Error parsing float FieldValue"
            );
            Box::new(f)
        }
        Rule::char => {
            let ch = value_string.chars().nth(1).expect(
                "Error parsing char FieldValue"
            );
            Box::new(ch as u8)
        },
        Rule::string => {
            let str = pair.into_inner().as_str();
            Box::new(str.to_string())
        },
        Rule::array => {
            let mut list = DynamicList::default();

            for value in pair.into_inner() {
                let array_val = parse_value(value);
                list.push_box(array_val);
            }

            Box::new(list)
        },
        Rule::range => {
            let i0 = value_string.find("..").unwrap();
            let i1 = value_string.rfind("..").unwrap() + 2;

            let start = &value_string[1..i0].parse::<i32>().expect(
                "Error parsing min range value"
            );
            let end = &value_string[i1..value_string.len() - 1].parse::<i32>().expect(
                "Error parsing max range value"
            );

            Box::new(Range::<i32> {
                start: *start,
                end: *end,
            })
        },
        _ => unreachable!()
    }
}


#[test]
fn test_insert() {
    
    #[derive(Default, Reflect)]
    struct TestStruct {
        i: i32,
    }

    let input = "aaa( TestStruct { i: 5 } )";
    let mut reg = PrefabRegistry::default();
    reg.register_component::<TestStruct>();

    let prefab = parse_prefab(input, &reg).expect("Error");

    {
        assert_eq!("aaa", prefab.name().unwrap());
        assert_eq!(1, prefab.components().len());
    }
    
    let comp = prefab.component("TestStruct").as_ref();

    //assert!(comp.is_some());

    //let comp = &comp.unwrap().reflect;

}

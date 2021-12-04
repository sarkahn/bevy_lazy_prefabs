use std::ops::Range;

use thiserror::Error;

use bevy::{
    prelude::*,
    reflect::{DynamicList, DynamicStruct, DynamicTuple, DynamicTupleStruct, Reflect},
};
use pest::{
    error::Error,
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::*;

use crate::dynamic_cast::*;
use crate::{
    prefab::*,
    registry::{PrefabRegistry, ReflectType, TypeInfo},
};

#[derive(Parser)]
#[grammar = "lazy_prefabs.pest"]
struct PrefabParser;

#[derive(Error, Debug)]
pub enum LoadPrefabError {
    #[error("Error reading prefab file")]
    PrefabFileReadError(#[from] std::io::Error),
    #[error("Error parsing component - {0} was not registered with the PrefabRegistry")]
    UnregisteredPrefabComponent(String),
    #[error("Pest error parsing prefab string")]
    PestParseError(#[from] Error<Rule>),
    #[error("Error parsing prefab value - unknown value rule {0}")]
    UnhandledPrefabValueRule(String),
    #[error("Error parsing prefab component - unknown component rule {0}")]
    UnhandledPrefabComponentRule(String),
}

pub fn parse_prefab(input: &str, registry: &mut PrefabRegistry) -> Result<Prefab, LoadPrefabError> {
    let parsed = match PrefabParser::parse(Rule::prefab, input) {
        Ok(parsed) => parsed,
        Err(e) => return Err(LoadPrefabError::PestParseError(e)),
    };

    let mut prefab_name = None;
    let mut components = Vec::new();

    for prefab_parse in parsed {
        match prefab_parse.as_rule() {
            Rule::prefab_name => {
                prefab_name = Some(prefab_parse.as_str().to_string());
            }
            // Type Name, Components
            Rule::component => {
                let component_parse = prefab_parse.into_inner();

                match parse_component(component_parse, registry) {
                    Ok(component) => components.push(component),
                    Err(e) => return Err(e),
                };
            }
            _ => {
                let str = format!("{:#?}", prefab_parse.as_rule());
                return Err(LoadPrefabError::UnhandledPrefabComponentRule(str));
            }
        }
    }

    Ok(Prefab::new(prefab_name, components))
}

fn parse_component(
    component_parse: Pairs<Rule>,
    registry: &mut PrefabRegistry,
) -> Result<PrefabComponent, LoadPrefabError> {
    let mut type_name = None;
    let mut fields = Vec::new();

    for component_parse in component_parse {
        match component_parse.as_rule() {
            Rule::type_name => {
                type_name = Some(component_parse.as_str().to_string());
            }
            Rule::component => {
                let nested_component =
                    parse_component(component_parse.into_inner(), registry).unwrap();
                fields.push(ReflectField::from(nested_component));
            }
            Rule::field => {
                let field = parse_field(component_parse.into_inner())?;
                fields.push(field);
            }
            _ => {
                let str = format!("{:#?}", component_parse.as_rule());
                return Err(LoadPrefabError::UnhandledPrefabComponentRule(str));
            },
        }
    }
    let type_name = type_name.unwrap();
    let type_info = match registry.type_info(&type_name) {
        Some(i) => i,
        None => return Err(LoadPrefabError::UnregisteredPrefabComponent(type_name)),
    };
    let root = build_root(type_info, fields);
    Ok(PrefabComponent::new(type_name.as_str(), root))
}

fn parse_field(mut field_parse: Pairs<Rule>) -> Result<ReflectField, LoadPrefabError> {
    let field_name = field_parse.next().unwrap().as_str();
    let value = parse_value(field_parse.next().unwrap())?;

    Ok(ReflectField {
        name: field_name.to_string(),
        value,
    })
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
        }
        ReflectType::TupleStruct => {
            let mut root = DynamicTupleStruct::default();
            for field in fields {
                root.insert_boxed(field.value);
            }
            Box::new(root)
        }
        ReflectType::Tuple => {
            let mut root = DynamicTuple::default();
            for field in fields {
                root.insert_boxed(field.value);
            }
            Box::new(root)
        }
        ReflectType::List => todo!(),
        ReflectType::Map => todo!(),
        ReflectType::Value => todo!(),
    }
}

fn parse_value(pair: Pair<Rule>) -> Result<Box<dyn Reflect>, LoadPrefabError> {
    let value_string = pair.as_str();
    match pair.as_rule() {
        Rule::int => {
            let num = value_string
                .parse::<i32>()
                .expect("Error parsing int FieldValue");
            Ok(Box::new(num))
        }
        Rule::float => {
            let f = value_string
                .parse::<f32>()
                .expect("Error parsing float FieldValue");
            Ok(Box::new(f))
        }
        Rule::char => {
            let ch = value_string
                .chars()
                .nth(1)
                .expect("Error parsing char FieldValue");
            Ok(Box::new(ch as u8))
        }
        Rule::string => {
            let str = pair.into_inner().as_str();
            Ok(Box::new(str.to_string()))
        }
        Rule::array => {
            let mut list = DynamicList::default();

            for value in pair.into_inner() {
                let array_val = parse_value(value)?;
                list.push_box(array_val);
            }

            Ok(Box::new(list))
        }
        Rule::range => {
            let i0 = value_string.find("..").unwrap();
            let i1 = value_string.rfind("..").unwrap() + 2;

            let start = &value_string[1..i0]
                .parse::<i32>()
                .expect("Error parsing min range value");
            let end = &value_string[i1..value_string.len() - 1]
                .parse::<i32>()
                .expect("Error parsing max range value");

            Ok(Box::new(Range::<i32> {
                start: *start,
                end: *end,
            }))
        }
        Rule::vec3 => {
            let mut v = Vec3::default();
            for field in pair.into_inner() {
                let (name, val) = parse_field(field.into_inner()).unwrap().into();
                let val = val.cast_ref::<f32>();
                match name.as_str() {
                    "x" => v.x = *val,
                    "y" => v.y = *val,
                    "z" => v.z = *val,
                    _ => {} // Error here?
                };
            }
            Ok(Box::new(v))
        }
        _ => {
            let str = format!("{:#?}", pair.as_rule());
            Err(LoadPrefabError::UnhandledPrefabValueRule(str))
        }
    }
}

#[cfg(test)]
mod test {
    use bevy::prelude::*;
    use pest::Parser;

    use crate::{parse::{PrefabParser, Rule, parse_value, parse_component}};
    use crate::registry::PrefabRegistry;

    #[test]
    fn vec_parse() {
        let input = "Vec3 { z: 3.0, x: 10.0 }";

        let mut reg = PrefabRegistry::default();
        
        reg.register_component::<Vec3>();

        let mut parse = PrefabParser::parse(Rule::vec3, input).unwrap();
        let parse = parse.next().unwrap();

        let mut v = Vec3::default();

        let dynamic = parse_value(parse).unwrap();

        v.apply(&*dynamic);

        assert_eq!(v.x, 10.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn transform_parse() {
        let mut reg = PrefabRegistry::default();

        reg.register_component::<Vec3>();
        reg.register_component::<Transform>();

        let input = "Transform { translation: Vec3 { y: 3.5, x: 10.5 } }";

        let mut parsed = PrefabParser::parse(Rule::component, input).unwrap();

        let parsed = parsed.next().unwrap().into_inner();

        let comp = parse_component(parsed, &mut reg).unwrap();

        let mut transform = Transform::default();

        transform.apply(comp.root());

        assert_eq!(transform.translation.y, 3.5);
        assert_eq!(transform.translation.x, 10.5);
    }
}
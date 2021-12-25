use bevy::{
    prelude::*,
    reflect::{
        DynamicList, DynamicStruct, DynamicTuple, DynamicTupleStruct, Reflect,
    },
};
use pest::{error::Error, iterators::Pair, Parser};
use pest_derive::*;
use std::{ops::Range, sync::Arc};
use thiserror::Error;

use crate::{
    dynamic_cast::*,
    prefab::PrefabBuildStep,
    registry::{ReflectType, TypeInfo, PrefabRegistry},
    prefab::*, 
};

#[derive(Parser)]
#[grammar = "lazy_prefabs.pest"]
struct PrefabParser;

/// A name/value pair representing a field on a type
#[derive(Debug)]
struct ReflectField {
    pub name: String,
    pub value: Box<dyn Reflect>,
}

impl From<PrefabComponent> for ReflectField {
    fn from(comp: PrefabComponent) -> Self {
        ReflectField {
            name: comp.type_name,
            value: comp.reflect,
        }
    }
}

#[derive(Error, Debug)]
pub enum LoadPrefabError {
    #[error("Pest error parsing prefab string.")]
    PestParseError(#[from] Error<Rule>),
    #[error("Error parsing prefab - unknown field rule: {0}.")]
    UnhandledPrefabFieldRule(String),
    #[error("Error parsing prefab - unknown component field rule: {0}.")]
    UnhandledPrefabComponentFieldRule(String),
    #[error("Error parsing component - {0} was not registered with the PrefabRegistry.")]
    UnregisteredPrefabComponent(String),
    #[error("Error parsing value type '{0}' from '{1}'.")]
    ValueParseError(String, String),
    #[error("Error parsing prefab - unknown value rule: {0}.")]
    UnhandledValueRule(String),
    #[error("Error reading prefab file.")]
    FileReadError(#[from] std::io::Error),
}

pub(crate) fn parse_prefab_string(
    input: &str,
    registry: &mut PrefabRegistry,
) -> Result<Prefab, LoadPrefabError> {

    let mut parsed = PrefabParser::parse(Rule::prefab, input)?;

    parse_prefab(parsed.next().unwrap(), registry)
}

fn parse_prefab(pair: Pair<Rule>, registry: &PrefabRegistry) -> Result<Prefab, LoadPrefabError> {
    let mut name = None;
    let mut steps = Vec::new();

    for field in pair.into_inner() {
        match field.as_rule() {
            Rule::type_name => {
                name = Some(field.as_str().to_string());
            }
            Rule::component => {
                let comp = parse_component(field, registry)?;
                steps.push(PrefabBuildStep::AddComponent(Arc::new(comp)));
            }
            Rule::command => {
                let command = parse_command(field)?;
                steps.push(PrefabBuildStep::RunCommand(Arc::new(command)));
            }
            _ => {
                let str = format!("{:#?}", field.as_rule());
                return Err(LoadPrefabError::UnhandledPrefabFieldRule(str));
            }
        }
    }

    Ok(Prefab { name, steps })
}

fn parse_component(
    pair: Pair<Rule>,
    registry: &PrefabRegistry,
) -> Result<PrefabComponent, LoadPrefabError> {
    let mut fields = Vec::new();

    let mut pairs = pair.into_inner();
    let type_name = pairs.next().unwrap().as_str();

    // Prefab fields
    for field in pairs {
        match field.as_rule() {
            Rule::component => {
                let nested_component = parse_component(field, registry).unwrap();
                fields.push(ReflectField::from(nested_component));
            }
            Rule::field => {
                let field = parse_field(field)?;
                fields.push(field);
            }
            _ => {
                let str = format!("{:#?}", field.as_rule());
                return Err(LoadPrefabError::UnhandledPrefabComponentFieldRule(str));
            }
        }
    }
    let t = registry
        .get_type_data(type_name)
        .ok_or_else(|| LoadPrefabError::UnregisteredPrefabComponent(type_name.to_string()))?;

    let comp = build_component(t, fields);

    Ok(PrefabComponent {
        type_name: type_name.to_string(),
        reflect: comp,
    })
}

fn build_component(type_info: &TypeInfo, fields: Vec<ReflectField>) -> Box<dyn Reflect> {
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

fn parse_field(field: Pair<Rule>) -> Result<ReflectField, LoadPrefabError> {
    let mut field = field.into_inner();
    let field_name = field.next().unwrap().as_str();
    let value = parse_value(field.next().unwrap())?;

    Ok(ReflectField {
        name: field_name.to_string(),
        value,
    })
}

fn parse_value(pair: Pair<Rule>) -> Result<Box<dyn Reflect>, LoadPrefabError> {
    let value_string = pair.as_str();
    match pair.as_rule() {
        Rule::int => {
            let num = value_string.parse::<i32>().map_err(|_| {
                LoadPrefabError::ValueParseError("i32".to_string(), value_string.to_string())
            })?;
            Ok(Box::new(num))
        }
        Rule::float => {
            let f = value_string.parse::<f32>().map_err(|_| {
                LoadPrefabError::ValueParseError("float".to_string(), value_string.to_string())
            })?;
            Ok(Box::new(f))
        }
        Rule::char => {
            let ch = value_string.chars().nth(1).ok_or_else(|| {
                LoadPrefabError::ValueParseError("char".to_string(), value_string.to_string())
            })?;
            Ok(Box::new(ch as u8))
        }
        Rule::string => {
            let str = parse_string(pair);
            Ok(Box::new(str))
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

            let start = &value_string[1..i0].parse::<i32>().map_err(|_| {
                LoadPrefabError::ValueParseError("range min".to_string(), value_string.to_string())
            })?;
            let end = &value_string[i1..value_string.len() - 1]
                .parse::<i32>()
                .map_err(|_| {
                    LoadPrefabError::ValueParseError(
                        "range max".to_string(),
                        value_string.to_string(),
                    )
                })?;

            Ok(Box::new(Range::<i32> {
                start: *start,
                end: *end,
            }))
        }
        Rule::vec3 => {
            let mut v = Vec3::default();
            for field in pair.into_inner() {
                let field = parse_field(field).unwrap();
                let name = field.name;
                let val = field.value.cast_ref::<f32>();
                match name.as_str() {
                    "x" => v.x = *val,
                    "y" => v.y = *val,
                    "z" => v.z = *val,
                    _ => {} // Error here?
                };
            }
            Ok(Box::new(v))
        }
        Rule::color => {
            let pair = pair.into_inner().next().unwrap();
            let value_string = pair.as_str();
            let col = match value_string {
                "RED" => Color::RED,
                "BLUE" => Color::BLUE,
                "GREEN" => Color::GREEN,
                "YELLOW" => Color::YELLOW,
                "PINK" => Color::PINK,
                _ => {
                    let str = format!("Color::{}", value_string);
                    return Err(LoadPrefabError::UnhandledValueRule(str));
                }
            };
            Ok(Box::new(col))
        }
        Rule::shape => {
            let shape = pair.into_inner().next().unwrap().as_str();
            Ok(Box::new(shape.to_string()))
        }
        _ => {
            let str = format!("{:#?}", pair.as_rule());
            Err(LoadPrefabError::UnhandledValueRule(str))
        }
    }
}

fn parse_string(pair: Pair<Rule>) -> String {
    let str = pair.as_str();
    str[1..str.len().saturating_sub(1)].to_string()
}

fn parse_command(pair: Pair<Rule>) -> Result<PrefabCommandData, LoadPrefabError> {
    let mut pairs = pair.into_inner();
    let command_name = pairs.next().unwrap().as_str().to_string();

    let mut properties = None;

    for field in pairs {
        let field = parse_field(field)?;
        let props = properties.get_or_insert(DynamicStruct::default());

        props.insert_boxed(field.name.as_str(), field.value);
    }

    Ok(PrefabCommandData {
        name: command_name,
        properties,
    })
}

#[cfg(test)]
mod test {
    use bevy::prelude::*;
    
    use pest::Parser;

    use crate::dynamic_cast::*;
    use crate::parse::parse_prefab;
    use crate::prefab::PrefabBuildStep;
    use crate::registry::PrefabRegistry;
    use crate::{
        dynamic_cast::GetValue,
        parse::{parse_component, parse_value, PrefabParser, Rule},
    };
    

    use super::{parse_command, parse_field, parse_string};

    #[test]
    fn command_parse() {
        let input = "DOSTUFF!(i: 10)";

        let parse = PrefabParser::parse(Rule::command, input)
            .unwrap()
            .next()
            .unwrap();

        let parsed = parse_command(parse).unwrap();

        let props = parsed.properties.unwrap();

        let i = *props.get::<i32>("i");

        assert_eq!(i, 10);
    }

    #[test]
    fn prefab_parse() {
        let input = "SomeName { dosomething!(), Visible, Draw }";
        let mut parsed = PrefabParser::parse(Rule::prefab, input).unwrap();
        let mut reg = PrefabRegistry::default();
        reg.register_type::<Visible>();
        reg.register_type::<Draw>();

        let prefab = parse_prefab(parsed.next().unwrap(), &mut reg).unwrap();

        assert_eq!(prefab.name, Some("SomeName".to_string()));

        match &prefab.steps[0] {
            PrefabBuildStep::AddComponent(_) => unreachable!(),
            PrefabBuildStep::RunCommand(command) => {
                assert_eq!(command.name, "dosomething");
            },
        }

        match &prefab.steps[1] {
            PrefabBuildStep::AddComponent(comp) => {
                assert_eq!(comp.type_name, "Visible");
            },
            PrefabBuildStep::RunCommand(_) => unreachable!(),
        }

        match &prefab.steps[2] {
            PrefabBuildStep::AddComponent(comp) => {
                assert_eq!(comp.type_name, "Draw");
            },
            PrefabBuildStep::RunCommand(_) => unreachable!(),
        }
    }

    #[test]
    fn char_parse() {
        let input = "'a'";
        let parse = PrefabParser::parse(Rule::value, input)
            .unwrap()
            .next()
            .unwrap();
        let parsed = parse_value(parse);
        assert!(parsed.is_ok());
        let val = *parsed.unwrap().downcast::<u8>().unwrap();
        assert_eq!(val as char, 'a');
    }

    #[test]
    fn color_parse() {
        let input = "Color::RED";
        let parse = PrefabParser::parse(Rule::color, input)
            .unwrap()
            .next()
            .unwrap();
        let parsed = parse_value(parse);
        let val = *parsed.unwrap().downcast::<Color>().unwrap();

        assert_eq!(Color::RED, val);
    }

    #[test]
    fn vec_parse() {
        let input = "Vec3 { z: 3.0, x: 10.0 }";

        let mut reg = PrefabRegistry::default();

        reg.register_type::<Vec3>();

        let parse = PrefabParser::parse(Rule::vec3, input)
            .unwrap()
            .next()
            .unwrap();

        let mut v = Vec3::default();

        let dynamic = parse_value(parse).unwrap();

        v.apply(&*dynamic);

        assert_eq!(v.x, 10.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn transform_parse() {
        let mut reg = PrefabRegistry::default();

        reg.register_type::<Vec3>();
        reg.register_type::<Transform>();

        let input = "Transform { translation: Vec3 { y: 3.5, x: 10.5 } }";

        let parsed = PrefabParser::parse(Rule::component, input)
            .unwrap()
            .next()
            .unwrap();

        let comp = parse_component(parsed, &mut reg).unwrap();

        let mut transform = Transform::default();

        transform.apply(&*comp.reflect);

        assert_eq!(transform.translation.y, 3.5);
        assert_eq!(transform.translation.x, 10.5);
    }

    #[test]
    fn string_parse() {
        let input = "\"Hello\"";
        let mut parsed = PrefabParser::parse(Rule::string, input).unwrap();
        let str = parse_string(parsed.next().unwrap());

        assert_eq!("Hello", str);
    }

    #[test]
    fn field_parse() {
        let input = "a: \"hi\"";

        let mut parse = PrefabParser::parse(Rule::field, input).unwrap();
        let field = parse_field(parse.next().unwrap()).unwrap();

        assert_eq!("a", field.name);
        assert_eq!("hi", field.value.cast_ref::<String>());
    }
}

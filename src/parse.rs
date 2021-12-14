use bevy::{
    prelude::*,
    reflect::{DynamicList, DynamicStruct, DynamicTuple, DynamicTupleStruct, Reflect, ReflectRef},
};
use pest::{error::Error, iterators::Pair, Parser};
use pest_derive::*;
use std::{any::Any, ops::Range};
use thiserror::Error;

use crate::{commands::PrefabCommand, dynamic_cast::*, registry::TypeInfo};
use crate::{prefab::*, registry::PrefabRegistry};

#[derive(Parser)]
#[grammar = "lazy_prefabs.pest"]
struct PrefabParser;

#[derive(Error, Debug)]
pub enum LoadPrefabError {
    #[error("Error reading prefab file.")]
    PrefabFileReadError(#[from] std::io::Error),
    #[error("Error parsing component - {0} was not registered with the PrefabRegistry.")]
    UnregisteredPrefabComponent(String),
    #[error("Pest error parsing prefab string.")]
    PestParseError(#[from] Error<Rule>),
    #[error("Error parsing prefab - unknown value rule: {0}.")]
    UnhandledValueRule(String),
    #[error("Error parsing prefab - unknown field rule: {0}.")]
    UnhandledPrefabFieldRule(String),
    #[error("Error parsing prefab - unknown material field: {0}.")]
    UnhandledMaterialRule(String),
    #[error("Error parsing prefab material - missing required field {0}.")]
    MissingMaterialField(String),
    #[error("Error parsing value type '{0}' from '{1}'.")]
    ValueParseError(String, String),
    #[error("Error parsing loader - unknown field rule {0}")]
    LoadParseError(String),
    #[error("Error parsing prefab - processor key {0} was not registered.")]
    UnregisteredProcessor(String),
    #[error(
        "Error parsing prefab - prefab component {0} is not properly reflected. Ensure custom
    components have the #[derive(Reflect)] and #[reflect(Component)] attributes."
    )]
    UnreflectedCompoent(String),
}

pub(crate) fn parse_prefab_string(
    input: &str,
    registry: &mut PrefabRegistry,
) -> Result<Prefab, LoadPrefabError> {
    let mut parsed = match PrefabParser::parse(Rule::prefab, input) {
        Ok(parsed) => parsed,
        Err(e) => return Err(LoadPrefabError::PestParseError(e)),
    };

    parse_prefab(parsed.next().unwrap(), registry)
}

fn parse_prefab(
    pair: Pair<Rule>,
    registry: &mut PrefabRegistry,
) -> Result<Prefab, LoadPrefabError> {
    let mut commands = Vec::new();

    let pair = pair.into_inner();
    let mut name = None;

    for field in pair {
        match field.as_rule() {
            Rule::type_name => {
                name = Some(field.as_str().to_string());
            }
            Rule::component => {
                let comp = parse_component(field, registry)?;
                commands.push(PrefabCommand::AddComponent(comp));
            }
            Rule::processor => {
                let processor = parse_processor(field, registry)?;

                commands.push(PrefabCommand::Processor(processor))
            }
            Rule::load => {
                let load = parse_load(field, registry)?;

                commands.push(PrefabCommand::LoadPrefab(load));
            }
            _ => {
                let str = format!("{:#?}", field.as_rule());
                return Err(LoadPrefabError::UnhandledPrefabFieldRule(str));
            }
        }
    }

    Ok(Prefab::new(name, commands))
}

fn parse_component(
    pair: Pair<Rule>,
    registry: &mut PrefabRegistry,
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
                return Err(LoadPrefabError::UnhandledPrefabFieldRule(str));
            }
        }
    }
    let type_info = match registry.type_info(type_name) {
        Some(i) => i,
        None => {
            return Err(LoadPrefabError::UnregisteredPrefabComponent(
                type_name.to_string(),
            ))
        }
    };
    let root = build_root(type_info, fields);
    let reflect = match type_info.registration.data::<ReflectComponent>() {
        Some(reflect) => reflect,
        None => return Err(LoadPrefabError::UnreflectedCompoent(type_name.to_string())),
    };

    let type_id = type_info.type_id();
    Ok(PrefabComponent::new(
        type_name,
        root,
        reflect.clone(),
        type_id,
    ))
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
                let (name, val) = parse_field(field).unwrap().into();
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

fn parse_processor(
    pair: Pair<Rule>,
    registry: &PrefabRegistry,
) -> Result<PrefabProcessorData, LoadPrefabError> {
    let mut pairs = pair.into_inner();
    let key = pairs.next().unwrap().as_str();

    let mut properties = None;

    for field in pairs {
        let field = parse_field(field)?;

        let properties = properties.get_or_insert(DynamicStruct::default());
        properties.insert_boxed(field.name.as_str(), field.value);
    }

    let processor = match registry.get_processor(key) {
        Some(processor) => processor,
        None => return Err(LoadPrefabError::UnregisteredProcessor(key.to_string())),
    }
    .clone();

    Ok(PrefabProcessorData::new(key, properties, processor))
}

fn parse_load(
    pair: Pair<Rule>,
    registry: &mut PrefabRegistry,
) -> Result<PrefabLoad, LoadPrefabError> {
    let mut pairs = pair.into_inner();

    let path = pairs.next().unwrap().as_str();

    // Ensure the prefab is loaded
    registry.load(path)?;

    let mut components = None;

    for field in pairs {
        match field.as_rule() {
            Rule::component => {
                let component = parse_component(field, registry)?;
                let components = components.get_or_insert(Vec::new());
                components.push(component);
            }
            Rule::processor => {}
            _ => return Err(LoadPrefabError::LoadParseError(field.as_str().to_string())),
        }
    }

    Ok(PrefabLoad::new(path))
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) enum ReflectType {
    Struct,
    TupleStruct,
    Tuple,
    List,
    Map,
    Value,
}

impl<'a> From<ReflectRef<'a>> for ReflectType {
    fn from(reflect: ReflectRef) -> Self {
        match reflect {
            ReflectRef::Struct(_) => ReflectType::Struct,
            ReflectRef::TupleStruct(_) => ReflectType::TupleStruct,
            ReflectRef::Tuple(_) => ReflectType::Tuple,
            ReflectRef::List(_) => ReflectType::List,
            ReflectRef::Map(_) => ReflectType::Map,
            ReflectRef::Value(_) => ReflectType::Value,
        }
    }
}

#[cfg(test)]
mod test {
    use bevy::prelude::*;
    use bevy::reflect::DynamicStruct;
    use pest::Parser;

    use crate::dynamic_cast::*;
    use crate::processor::ColorMaterialProcessor;
    use crate::registry::PrefabRegistry;
    use crate::{
        dynamic_cast::GetValue,
        parse::{parse_component, parse_value, PrefabParser, Rule},
    };
    use bevy::ecs::reflect::ReflectComponent;

    use super::{parse_field, parse_load, parse_processor, parse_string};

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

        transform.apply(comp.root());

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
    fn processor_parse() {
        let input = "processor!( ColorMaterial { 
            texture_path: \"Alien.png\", 
            color: Color::RED,
        })";
        let mut reg = PrefabRegistry::default();
        reg.init_processor::<ColorMaterialProcessor>();
        let mut parse = PrefabParser::parse(Rule::processor, input).unwrap();
        let data = parse_processor(parse.next().unwrap(), &reg).unwrap();

        assert_eq!("ColorMaterial", data.key());
        assert!(data.properties().is_some());
        let props = data.properties().unwrap();

        assert_eq!("Alien.png", props.get::<String>("texture_path").as_str());
        assert_eq!(Color::RED, *props.get::<Color>("color"));
    }

    #[test]
    fn field_parse() {
        let input = "a: \"hi\"";

        let mut parse = PrefabParser::parse(Rule::field, input).unwrap();
        let field = parse_field(parse.next().unwrap()).unwrap();

        assert_eq!("a", field.name);
        assert_eq!("hi", field.value.cast_ref::<String>());
    }

    #[derive(Reflect, Default)]
    #[reflect(Component)]
    struct TestComponentA;

    #[derive(Reflect, Default)]
    #[reflect(Component)]
    struct TestComponentB {
        x: i32,
    }

    #[test]
    fn load_parse() {
        let input = "load!(test.prefab)";

        let mut reg = PrefabRegistry::default();
        reg.register_type::<TestComponentA>();
        reg.register_type::<TestComponentB>();

        let mut parsed = PrefabParser::parse(Rule::load, input).unwrap();
        let load = parse_load(parsed.next().unwrap(), &mut reg).unwrap();

        assert_eq!("test.prefab", load.path());

        let prefab = reg.try_load(load.path()).unwrap();

        match &prefab.commands()[0] {
            crate::commands::PrefabCommand::AddComponent(comp) => {
                assert_eq!(comp.name(), "TestComponentA");
            },
            _ => panic!()
        };
        match &prefab.commands()[1] {
            crate::commands::PrefabCommand::AddComponent(comp) => {
                assert_eq!(comp.name(), "TestComponentB");
                let comp = comp.root().cast_ref::<DynamicStruct>();
                assert_eq!(35, *comp.get::<i32>("x"));
            },
            _ => panic!()
        };
    }

    #[test]
    fn parse_mesh() {
        let input = "shape::Cube { size: 15.0 }";
        let parsed = PrefabParser::parse(Rule::value, input).unwrap().next().unwrap();
        let field = parse_value(parsed);
    }
}

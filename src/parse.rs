use std::ops::Range;

use thiserror::Error;

use bevy::{
    prelude::*,
    reflect::{DynamicList, DynamicStruct, DynamicTuple, DynamicTupleStruct, Reflect, ReflectRef},
};
use pest::{
    error::Error,
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::*;

use crate::{bundle::PrefabBundle, dynamic_cast::*, registry::TypeInfo, PrefabMaterial};
use crate::{prefab::*, registry::PrefabRegistry};

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
    #[error("Error parsing prefab - unknown value rule {0}")]
    UnhandledValueRule(String),
    #[error("Error parsing prefab - unknown field rule {0}")]
    UnhandledPrefabFieldRule(String),
    #[error("Error parsing prefab - unknown material field {0}")]
    UnhandledMaterialRule(String),
    #[error("Error parsing prefab material - missing required field {0}")]
    MissingMaterialField(String),
    #[error("Error parsing value type {0} ({1})")]
    ValueParseError(String, String),
}

pub fn parse_prefab(input: &str, registry: &mut PrefabRegistry) -> Result<Prefab, LoadPrefabError> {
    let parsed = match PrefabParser::parse(Rule::prefab, input) {
        Ok(parsed) => parsed,
        Err(e) => return Err(LoadPrefabError::PestParseError(e)),
    };

   // println!("PARSING PREFAB");

    let mut prefab_name = None;
    let mut components = Vec::new();
    let mut bundles = None;
    let mut material = None;
    let mut processors = None;

    // Prefab fields
    for field in parsed {
        match field.as_rule() {
            Rule::prefab_name => {
                prefab_name = Some(field.as_str().to_string());
            }
            // Type Name, Components
            Rule::component => {
                //println!("Parsing material {}", field.as_str());
                let comp = parse_component(field, registry)?;
                components.push(comp);
            }
            // Rule::bundle => {
            //     //println!("Parsing bundle");
            //     let bundles = bundles.get_or_insert(Vec::new());

            //     let bundle = parse_bundle(field, registry)?;

            //     bundles.push(bundle);
            // },
            Rule::processor => {
                let processor = parse_processor(field)?;
                let processors = processors.get_or_insert(Vec::new());
                processors.push(processor);
            },
            _ => {
                let str = format!("{:#?}", field.as_rule());
                return Err(LoadPrefabError::UnhandledPrefabFieldRule(str));
            }
        }
    }

    Ok(Prefab::new(prefab_name, components, bundles, material, processors))
}

fn parse_component(
    pair: Pair<Rule>,
    registry: &mut PrefabRegistry,
) -> Result<PrefabComponent, LoadPrefabError> {
    let mut fields = Vec::new();

    let mut pairs = pair.into_inner();
    let type_name = pairs.next().unwrap().as_str();
    //println!("Type name {}", type_name);

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
            },
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
    Ok(PrefabComponent::new(type_name, root))
}

fn parse_field(mut field: Pair<Rule>) -> Result<ReflectField, LoadPrefabError> {
    let mut field = field.into_inner();
    //println!("FIELD CONTENT: {} ", field.as_str());
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
            let str = pair.as_str();
            //println!("Parsing string {}", str);
            let str = &str[1..str.len().saturating_sub(1)];
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

fn parse_bundle(
    pair: Pair<Rule>,
    _registry: &mut PrefabRegistry,
) -> Result<PrefabBundle, LoadPrefabError> {
    let mut pair = pair.into_inner();

    // BundleName
    let bundle_type_name = pair.next().unwrap().as_str();

    Ok(PrefabBundle::new(bundle_type_name, None))
}

fn parse_material(pair: Pair<Rule>) -> Result<PrefabMaterial, LoadPrefabError> {
    let mut pairs = pair.into_inner();

    // Type name = as_str()
    pairs.next().unwrap();

    let pairs = pairs.next();

    let mut texture_path = None;
    let mut loader_key = None;
    let mut properties = None;

    if let Some(pairs) = pairs {
        for field in pairs.into_inner() {
            match field.as_rule() {
                Rule::material_texture_path => {
                    let field = field.into_inner().next().unwrap();
                    texture_path = Some(parse_string(field));
                }
                Rule::material_loader_key => {
                    let field = field.into_inner().next().unwrap();
                    loader_key = Some(parse_string(field))
                }
                Rule::field => {
                    let mut field = field.into_inner();
                    let name = field.next().unwrap().as_str();
                    let val = parse_value(field.next().unwrap());

                    let props = properties.get_or_insert(DynamicStruct::default());
                    props.insert_boxed(name, val.unwrap());
                }
                _ => {
                    let str = format!("{:#?}", field.as_rule());
                    return Err(LoadPrefabError::UnhandledMaterialRule(str));
                }
            }
        }
    }

    if texture_path.is_none() {
        return Err(LoadPrefabError::MissingMaterialField(
            "texture_path".to_string(),
        ));
    }

    if loader_key.is_none() {
        return Err(LoadPrefabError::MissingMaterialField(
            "loader_key".to_string(),
        ));
    }

    Ok(PrefabMaterial::new(
        texture_path.unwrap().as_str(),
        loader_key.unwrap().as_str(),
        properties,
    ))
}

fn parse_processor(pair: Pair<Rule>) -> Result<PrefabProcessorData,LoadPrefabError>
{
    let mut pairs = pair.into_inner();
    let key = pairs.next().unwrap().as_str();

    let mut properties = None;

    for field in pairs {
        let field = parse_field(field)?;
        
        let properties = properties.get_or_insert(DynamicStruct::default());
        properties.insert_boxed(field.name.as_str(), field.value);
    }

    Ok(PrefabProcessorData::new(
        key,
        properties
    ))
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ReflectType {
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
    use pest::Parser;

    use crate::dynamic_cast::DynamicCast;
    use crate::registry::PrefabRegistry;
    use crate::{
        dynamic_cast::GetValue,
        parse::{parse_component, parse_value, PrefabParser, Rule},
    };

    use super::{parse_material, parse_processor, parse_field};

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

    #[derive(Reflect, Default)]
    struct TestStruct {
        i: i32,
    }

    // #[test]
    // fn prefab_bundle_parse() {
    //     let mut reg = PrefabRegistry::default();

    //     reg.register_type::<TestStruct>();

    //     let input = "WithBundle ( bundle!(SpriteBundle), TestStruct )";
    //     let prefab = parse_prefab(input, &mut reg).unwrap();

    //     assert!(prefab.bundles().is_some());
    //     let bundles = prefab.bundles().unwrap();

    //     assert_eq!(bundles[0].name(), "SpriteBundle");
    // }

    // fn bundle_with_fields_parse() {
    //     let input = "bundle!(SpriteBundle {
    //         Handle<ColorMaterial> {
    //             color = Color::RED,
    //             texture_path = \"alien.png\"
    //             loader_key = \"ColorMaterial\"
    //         }
    //     })";

    //     let mut registry = PrefabRegistry::default();

    //     let mut parse = PrefabParser::parse(Rule::bundle, input).unwrap();
    //     let parse = parse_bundle(parse.next().unwrap(), &mut registry).unwrap();

    //     assert!(parse.components().is_some());

    //     let components = parse.components().unwrap();
    //     assert_eq!(1, components.len());

    //     let comp = &components[0];

    //     //assert_eq!(Color::RED, parse.properties().unwrap().get::<Color>("color"));
    // }

    // #[test]
    // fn material_parse() {
    //     let input = "Handle<ColorMaterial> {
    //         texture_path: \"alien.png\",
    //         loader_key: \"ColorMaterial\",
    //         color: Color::BLUE,
    //     }";

    //     let parsed = PrefabParser::parse(Rule::material, input)
    //         .unwrap()
    //         .next()
    //         .unwrap();
    //     let res = parse_material(parsed).unwrap();

    //     assert_eq!("alien.png", res.texture_path());
    //     assert_eq!("ColorMaterial", res.loader_key());

    //     assert!(res.properties().is_some());

    //     let col = res.properties().unwrap().get::<Color>("color");

    //     assert_eq!(Color::BLUE, *col);
    // }

    #[test]
    fn processor_parse() {
        let input = "processor!( ColorMaterialProcessor { 
            texture_path: \"Alien.png\", 
            color: Color::RED,
        })";

        let mut parse = PrefabParser::parse(Rule::processor, input).unwrap();
        let data = parse_processor(parse.next().unwrap()).unwrap();

        assert_eq!("ColorMaterialProcessor", data.key());
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
}

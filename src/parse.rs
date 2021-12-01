
use thiserror::Error;

use bevy::reflect::{Reflect, DynamicStruct};
use pest::{Parser, error::Error, iterators::{Pair, Pairs}};
use pest_derive::*;

use crate::{prefab::{Prefab, PrefabComponent}, registry::PrefabRegistry};
use crate::dynamic_cast::*;

#[derive(Parser)]
#[grammar = "lazy_prefabs.pest"]
struct PrefabParser;

#[derive(Error, Debug)]
pub enum PrefabParserError {
    #[error("Error parsing component - {0} not registered")]
    UnregisteredComponent(String),
    #[error("Error parsing prefab")]
    ParserError(#[from] Error<Rule>),
}


pub fn parse<'a>(input: &str, registry: &'a PrefabRegistry) -> Result<Prefab, PrefabParserError> {

    let mut parsed = match PrefabParser::parse(Rule::prefab, input) {
        Ok(parsed) => parsed,
        Err(e) => return Err(PrefabParserError::ParserError(e)),
    };

    //println!("Parsed: {:#?}", parse_prefab);

    let mut prefab = Prefab::default();

    //let start = parse_prefab.next().unwrap();
    //println!("Start: {:#?}", start);

    for pair in parsed {
        match pair.as_rule() {
            Rule::prefab_name => {
                prefab.name = Some(pair.as_str().to_string());
            },
            Rule::component => {
                println!("COMPONENT");
            },
            _ => unreachable!()
        }
    }

    // if start.as_rule() == Rule::prefab_name {
    //     let prefab_name = parse_prefab.next().unwrap().as_str();
    //     println!("FOUND PREFAB NAME");
    
    //     if !prefab_name.is_empty() {
    //         prefab.name = Some(String::from(prefab_name));
    //     }
    // }

    // //println!("Parse: {:#?}", parse_prefab.next().unwrap());

    // let components = &mut prefab.components;

    // for parsed in parse_prefab {
    //     println!("Parsing component");
    //     let mut parsed = parsed.into_inner();
    //     let type_name = parsed.next().unwrap().as_str();

    //     let mut component = match registry.instance_clone(type_name) {
    //         Some(component) => component,
    //         None => return Err(PrefabParserError::UnregisteredComponent(String::from(type_name))),
    //     };

    //     parse_component(&mut component, parsed);

    //     // for parse_field in parsed {
    //     //     let mut parse_field = parse_field.into_inner();

    //     //     let field_name = parse_field.next().unwrap().as_str();
    //     //     let field_value = parse_field.next().unwrap();
            
    //     //     //let fields = &mut component.fields.get_or_insert(Vec::new());

    //     //     //let value = parse_value(field_value);

    //     //     // fields.push(PrefabComponentField {
    //     //     //     name: String::from(field_name),
    //     //     //     value: value,
    //     //     // });
    //     // }

    //     components.push(PrefabComponent {
    //         name: type_name.to_string(),
    //         reflect: component,
    //     });
    // }

//         components.push(component);
    Ok(prefab)
}

fn parse_component(component: &mut Box<dyn Reflect>, parsed_component: Pairs<Rule>) {
    println!("Parsing component");
    for parsed_field in parsed_component {
        let mut parsed_field = parsed_field.into_inner();

        let field_name = parsed_field.next().unwrap().as_str();
        let field_value = parsed_field.next().unwrap();

        parse_field(component, field_name, field_value);
        
        //let fields = &mut component.fields.get_or_insert(Vec::new());

        //let value = parse_value(field_value);

        // fields.push(PrefabComponentField {
        //     name: String::from(field_name),
        //     value: value,
        // });
    }
}

fn parse_field(component: &mut Box<dyn Reflect>, field_name: &str, pair: Pair<Rule>) {
    //print!("  {} : ", field_name);
    let str = pair.as_str();
    match pair.as_rule() {
        Rule::int => {
            let num = str.parse::<i32>().expect(
                "Error parsing int FieldValue"
            );
            insert_int(component, field_name, num );
        },
        Rule::float => {
            let f = str.parse::<f32>().expect(
                "Error parsing float FieldValue"
            );
            
        }
        Rule::char => {
            let ch = str.chars().nth(1).expect(
                "Error parsing char FieldValue"
            );
            
        },
        Rule::string => {
            let str = pair.into_inner().as_str();
            
        },
        Rule::array => {
            // let mut vec = Vec::new();
            // for value in pair.into_inner() {
            //     vec.push(parse_value(value));
            // }

            
        },
        Rule::range => {
            let i0 = str.find("..").unwrap();
            let i1 = str.rfind("..").unwrap() + 2;

            let min = &str[1..i0].parse::<i32>().expect(
                "Error parsing min range value"
            );
            let max = &str[i1..str.len() - 1].parse::<i32>().expect(
                "Error parsing max range value"
            );

            
        },
        _ => unreachable!()
    }
}

fn insert_int(component: &mut Box<dyn Reflect>, field_name: &str, value: i32) {
    match component.reflect_mut() {
        bevy::reflect::ReflectMut::Struct(_) => {
            println!("INSERTING INT");
            let component = component.cast_mut::<DynamicStruct>();
            component.insert(field_name, value);
        },
        bevy::reflect::ReflectMut::TupleStruct(t) => todo!(),
        bevy::reflect::ReflectMut::Tuple(t) => todo!(),
        bevy::reflect::ReflectMut::List(l) => todo!(),
        bevy::reflect::ReflectMut::Map(m) => todo!(),
        bevy::reflect::ReflectMut::Value(v) => todo!(),
    }
}

#[derive(Default, Reflect)]
struct TestStruct {
    i: i32,
}

#[test]
fn test_insert() {
    let input = "aaa( TestStruct { i: 5 } )";
    let mut reg = PrefabRegistry::default();
    reg.register_component::<TestStruct>();

    let prefab = parse(input, &reg).expect("Error");

    if let Some(name) = prefab.name {
        println!("Prefab name: {}", name);
    }
    
    //let comp = prefab.component("TestStruct");

    //assert!(comp.is_some());

    //let comp = &comp.unwrap().reflect;

}

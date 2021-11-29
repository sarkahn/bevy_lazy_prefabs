use std::ops::Range;

use pest::{Parser, error::Error, iterators::Pair};
use pest_derive::*;

use crate::prefab::{FieldValue, Prefab, PrefabComponent, PrefabComponentField};

#[derive(Parser)]
#[grammar = "lazy_prefabs.pest"]
struct PrefabParser;

fn parse(input: &str) -> Result<Prefab, Error<Rule>> {

    let mut prefab = Prefab::default();

    let mut parse_prefab = PrefabParser::parse(Rule::prefab, input)?;

    let prefab_name = parse_prefab.next().unwrap().as_str();

    if !prefab_name.is_empty() {
        prefab.name = Some(String::from(prefab_name));
    }

    let components = &mut prefab.components;

    for parse_component in parse_prefab {
        let mut parse_component = parse_component.into_inner();
        let type_name = parse_component.next().unwrap().as_str();

        let mut component = PrefabComponent::default();
        component.name = String::from(type_name);

        for parse_field in parse_component {
            let mut parse_field = parse_field.into_inner();

            let field_name = parse_field.next().unwrap().as_str();
            let field_value = parse_field.next().unwrap();
            
            let fields = &mut component.fields.get_or_insert(Vec::new());

            let value = parse_value(field_value);

            fields.push(PrefabComponentField {
                name: String::from(field_name),
                value: value,
            });
        }

        components.push(component);
    }

    //println!("{:#?}", prefab);

    Ok(prefab)
}

fn parse_value(pair: Pair<Rule>) -> FieldValue {
    //print!("  {} : ", field_name);
    let str = pair.as_str();
    match pair.as_rule() {
        Rule::int => {
            let num = str.parse::<i32>().unwrap();
            FieldValue::Int(num)
        },
        Rule::float => {
            let f = str.parse::<f32>().unwrap();
            FieldValue::Float(f)
        }
        Rule::char => {
            let ch = str.chars().nth(1).unwrap();
            FieldValue::Char(ch)
        },
        Rule::string => {
            let str = pair.into_inner().as_str();
            FieldValue::String(String::from(str))
        },
        Rule::array => {
            let mut vec = Vec::new();
            for value in pair.into_inner() {
                vec.push(parse_value(value));
            }

            FieldValue::Vec(vec)
        },
        Rule::range => {
            let i0 = str.find("..").unwrap();
            let i1 = str.rfind("..").unwrap() + 2;

            let min = &str[1..i0];
            let max = &str[i1..str.len() - 1];

            println!("SUBSTRING {} : {}", min, max);

            FieldValue::Int(0)
        },
        _ => unreachable!()
    }
}
 

#[test]
fn array_test() {
    let arr_string = r#"[1, "hi", 'q', (5..10), [0,1,2], 12.7, ],"#;
    let arr = PrefabParser::parse(Rule::array, arr_string).unwrap().next().unwrap();

    let mut values = arr.into_inner();

    let val = parse_value(values.next().unwrap());
    assert_eq!( val.as_int().unwrap(), 1 );

    let val = parse_value(values.next().unwrap());
    //assert_eq!( val.as_string().unwrap(), "hi");

    let val = parse_value(values.next().unwrap());
    assert_eq!( val.as_char().unwrap(), 'q');
    
    let val = parse_value(values.next().unwrap());
    print!("{:#?}", val);
    //assert_eq!( val.as_range().unwrap(), &(5..10) );
}

#[test]
fn range_test() {
    let range_string = "(15..10)";
    let parse = PrefabParser::parse(Rule::range, range_string).unwrap().next().unwrap();
    let val = parse_value(parse);
    print!("{:#?}", val);
}

#[test]
fn string_test() {
    let parsed = PrefabParser::parse(Rule::string, "HI").unwrap();
    println!("{}", parsed);
}



#[test]
fn test_parser() {
    let prefab_string = "Test (
        Position {
            x: 10,
            y: 15,
        },
        Movable,
        Renderable {
            glyph: '@',
        },
    )";
    let prefab = parse(prefab_string).unwrap();

    assert!(prefab.name.is_some());
    assert_eq!(prefab.name.unwrap(), "Test");

    let components = &prefab.components;

    assert_eq!(components.len(), 3);
    assert_eq!(components[0].name, "Position");
    assert_eq!(components[1].name, "Movable");
    assert_eq!(components[2].name, "Renderable");

    let pos = &components[0];

    assert!(pos.fields.is_some());
    let pos_field = pos.fields.as_ref().unwrap();

    assert_eq!(pos_field.len(), 2);

    assert_eq!(pos_field[0].name, "x");
    assert_eq!(pos_field[1].name, "y");

    assert!( matches!(pos_field[0].value, FieldValue::Int(10)) );
    assert!( matches!(pos_field[1].value, FieldValue::Int(15)) );

    let renderable = &components[2];
    assert!(renderable.fields.is_some());
    let renderable_fields = renderable.fields.as_ref().unwrap();

    assert_eq!(renderable_fields.len(), 1);
    assert_eq!(renderable_fields[0].name, "glyph");

    assert!( matches!(&renderable_fields[0].value, FieldValue::Char('@')));
}

#[cfg(test)]
mod test {
    use pest::{Parser, error::Error, iterators::Pair};
    use pest_derive::*;

    use crate::prefab::FieldValue;

    #[derive(Parser)]
    #[grammar = "lazy_prefabs.pest"]
    struct PrefabParser;

    use super::{parse, parse_value};

    


}
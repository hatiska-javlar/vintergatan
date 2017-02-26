use rustc_serialize::json::{Array, Json, Object};

use common::{Id, ParseCommandError, ParseCommandResult, PlayerId};

type Result<T> = ParseCommandResult<T>;

pub fn join<S: ToString>(vec: Vec<S>, sep: &str) -> String {
    vec
        .iter()
        .fold("".to_string(), |a, b| if a.len() > 0 { a + sep } else { a } + &b.to_string())
}

pub fn parse_json(string: &str) -> Result<Json> {
    Json::from_str(string)
        .map_err(ParseCommandError::ParserError)
}

pub fn parse_json_as_object(json: &Json) -> Result<&Object> {
    json.as_object()
        .ok_or(incompatible_type_error(""))
}

pub fn parse_string_from_json_object<'a>(object: &'a Object, property: &str) -> Result<&'a str> {
    parse_value_from_json_object(object, property)?
        .as_string()
        .ok_or(incompatible_type_error(property))
}

pub fn parse_object_from_json_object<'a>(object: &'a Object, property: &str) -> Result<&'a Object> {
    parse_value_from_json_object(object, property)?
        .as_object()
        .ok_or(incompatible_type_error(property))
}

pub fn parse_array_from_json_object<'a>(object: &'a Object, property: &str) -> Result<&'a Array> {
    parse_value_from_json_object(object, property)?
        .as_array()
        .ok_or(incompatible_type_error(property))
}

pub fn parse_player_id_from_json_object(object: &Object, property: &str) -> Result<PlayerId> {
    parse_u64_from_json_object(object, property)
        .map(|player_id| player_id as PlayerId)
}

pub fn parse_option_player_id_from_json_object(object: &Object, property: &str) -> Result<Option<PlayerId>> {
    let option_player_id = parse_value_from_json_object(object, property)?
        .as_u64()
        .map(|player_id| player_id as PlayerId);

    Ok(option_player_id)
}

pub fn parse_id_from_json_object(object: &Object, property: &str) -> Result<Id> {
    parse_u64_from_json_object(object, property)
        .map(|id| id as Id)
}

pub fn parse_u64_from_json_object(object: &Object, property: &str) -> Result<u64> {
    parse_value_from_json_object(object, property)?
        .as_u64()
        .ok_or(incompatible_type_error(property))
}

pub fn parse_f64_from_json_object(object: &Object, property: &str) -> Result<f64> {
    parse_value_from_json_object(object, property)?
        .as_f64()
        .ok_or(incompatible_type_error(property))
}

fn parse_value_from_json_object<'a>(object: &'a Object, property: &str) -> Result<&'a Json> {
    object.get(property)
        .ok_or(missed_property_error(property))
}

fn missed_property_error(property: &str) -> ParseCommandError {
    ParseCommandError::MissedProperty(property.to_string())
}

fn incompatible_type_error(property: &str) -> ParseCommandError {
    ParseCommandError::IncompatibleType(property.to_string())
}
use std::collections::HashMap;

use rustc_serialize::json::{Json, Object};

use client::planet::Planet;
use client::player::Player;
use client::squad::Squad;
use common::{Id, PlayerId, ParseCommandError, ParseCommandResult, Position};
use common::utils::json;

type Result<T> = ParseCommandResult<T>;

type ProcessCommandTuple = (
    HashMap<Id, Planet>,
    HashMap<PlayerId, Player>,
    HashMap<Id, Squad>,
    PlayerId,
    f64
);

pub fn parse_command(string: &str) -> Result<(String, Object)> {
    let json = json::parse_json(string)?;
    let params = json::parse_json_as_object(&json)?;

    let command = json::parse_string_from_json_object(params, "action")?;
    let data = json::parse_object_from_json_object(params, "data")?;

    return Ok((command.to_string(), data.to_owned()));
}

pub fn parse_process_command_data(data: &Object) -> Result<ProcessCommandTuple> {
    let process_command_tuple = (
        parse_planets(data)?,
        parse_players(data)?,
        parse_squads(data)?,
        json::parse_player_id_from_json_object(data, "id")?,
        json::parse_f64_from_json_object(data, "gold")?
    );

    return Ok(process_command_tuple);
}

pub fn parse_state_data(data: &Object) -> Result<String> {
    let state = json::parse_string_from_json_object(data, "state")?;

    return Ok(state.to_string());
}

pub fn format_ready_command() -> String {
    format!(r#"{{"action":"ready","data":{{}}}}"#)
}

pub fn format_squad_spawn_command(planet_id: Id) -> String {
    format!(
        r#"{{"action":"squad_spawn","data":{{"planet_id":{}}}}}"#,
        planet_id
    )
}

pub fn format_squad_move_command(squad_id: Id, x: f64, y: f64, cut_count: Option<u64>) -> String {
    format!(
        r#"{{"action":"squad_move","data":{{"squad_id":{},"x":{},"y":{},"cut_count":{}}}}}"#,
        squad_id,
        x,
        y,
        cut_count.map_or("null".to_string(), |count| count.to_string())
    )
}

fn parse_planets(params: &Object) -> Result<HashMap<Id, Planet>> {
    let planets_json_array = json::parse_array_from_json_object(params, "planets")?;

    let mut planets = HashMap::new();
    for planet_json in planets_json_array.into_iter() {
        let planet_json_object = json::parse_json_as_object(planet_json)?;

        let planet_id = json::parse_id_from_json_object(planet_json_object, "id")?;
        let x = json::parse_f64_from_json_object(planet_json_object, "x")?;
        let y = json::parse_f64_from_json_object(planet_json_object, "y")?;

        let owner = json::parse_option_player_id_from_json_object(planet_json_object, "owner")?;

        let planet = Planet::new(planet_id, Position(x, y), owner);
        planets.insert(planet_id, planet);
    }

    Ok(planets)
}

fn parse_players(params: &Object) -> Result<HashMap<PlayerId, Player>> {
    let players_json_array = json::parse_array_from_json_object(params, "players")?;

    let mut players = HashMap::new();
    for player_json in players_json_array.into_iter() {
        let player_json_object = json::parse_json_as_object(player_json)?;

        let player_id = json::parse_player_id_from_json_object(player_json_object, "id")?;
        let player_name = json::parse_string_from_json_object(player_json_object, "name")?;
        let player_state = json::parse_string_from_json_object(player_json_object, "state")?;

        let player = Player::new(player_id, player_name.to_string(), player_state.to_string());
        players.insert(player_id, player);
    }

    Ok(players)
}

fn parse_squads(params: &Object) -> Result<HashMap<Id, Squad>> {
    let squads_json_array = json::parse_array_from_json_object(params, "squads")?;

    let mut squads = HashMap::new();
    for squad_json in squads_json_array.into_iter() {
        let squad_json_object = json::parse_json_as_object(squad_json)?;

        let squad_id = json::parse_id_from_json_object(squad_json_object, "id")?;
        let owner = json::parse_player_id_from_json_object(squad_json_object, "owner")?;
        let x = json::parse_f64_from_json_object(squad_json_object, "x")?;
        let y = json::parse_f64_from_json_object(squad_json_object, "y")?;
        let count = json::parse_u64_from_json_object(squad_json_object, "count")?;

        let squad = Squad::new(squad_id, owner, Position(x, y), count);
        squads.insert(squad_id, squad);
    }

    Ok(squads)
}
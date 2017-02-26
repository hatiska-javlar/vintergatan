use std::collections::HashMap;

use rustc_serialize::json::{Json, Object};

use client::planet::Planet;
use client::player::Player;
use client::squad::Squad;
use common::{Id, PlayerId, ParseCommandError, ParseCommandResult, Position, utils};

type Result<T> = ParseCommandResult<T>;

type ProcessCommandTuple = (
    HashMap<Id, Planet>,
    HashMap<PlayerId, Player>,
    HashMap<Id, Squad>,
    PlayerId,
    f64
);

pub fn parse_process_command(string: &str) -> Result<ProcessCommandTuple> {
    let json = utils::parse_json(string)?;
    let params = utils::parse_json_as_object(&json)?;

    let process_command_tuple = (
        parse_planets(params)?,
        parse_players(params)?,
        parse_squads(params)?,
        utils::parse_player_id_from_json_object(params, "id")?,
        utils::parse_f64_from_json_object(params, "gold")?
    );

    return Ok(process_command_tuple);
}

pub fn format_squad_spawn_command(planet_id: Id) -> String {
    format!(
        r#"{{"action":"squad_spawn","data":{{"planet_id":{}}}}}"#,
        planet_id
    )
}

pub fn format_squad_move_command(squad_id: Id, x: f64, y: f64) -> String {
    format!(
        r#"{{"action":"squad_move","data":{{"squad_id":{},"x":{},"y":{}}}}}"#,
        squad_id,
        x,
        y
    )
}

fn parse_planets(params: &Object) -> Result<HashMap<Id, Planet>> {
    let planets_json_array = utils::parse_array_from_json_object(params, "planets")?;

    let mut planets = HashMap::new();
    for planet_json in planets_json_array.into_iter() {
        let planet_json_object = utils::parse_json_as_object(planet_json)?;

        let planet_id = utils::parse_id_from_json_object(planet_json_object, "id")?;
        let x = utils::parse_f64_from_json_object(planet_json_object, "x")?;
        let y = utils::parse_f64_from_json_object(planet_json_object, "y")?;

        let owner = utils::parse_option_player_id_from_json_object(planet_json_object, "owner")?;

        let planet = Planet::new(planet_id, Position(x, y), owner);
        planets.insert(planet_id, planet);
    }

    Ok(planets)
}

fn parse_players(params: &Object) -> Result<HashMap<PlayerId, Player>> {
    let players_json_array = utils::parse_array_from_json_object(params, "players")?;

    let mut players = HashMap::new();
    for player_json in players_json_array.into_iter() {
        let player_json_object = utils::parse_json_as_object(player_json)?;

        let player_id = utils::parse_player_id_from_json_object(player_json_object, "id")?;

        let player = Player::new(player_id);
        players.insert(player_id, player);
    }

    Ok(players)
}

fn parse_squads(params: &Object) -> Result<HashMap<Id, Squad>> {
    let squads_json_array = utils::parse_array_from_json_object(params, "squads")?;

    let mut squads = HashMap::new();
    for squad_json in squads_json_array.into_iter() {
        let squad_json_object = utils::parse_json_as_object(squad_json)?;

        let squad_id = utils::parse_id_from_json_object(squad_json_object, "id")?;
        let owner = utils::parse_player_id_from_json_object(squad_json_object, "owner")?;
        let x = utils::parse_f64_from_json_object(squad_json_object, "x")?;
        let y = utils::parse_f64_from_json_object(squad_json_object, "y")?;
        let count = utils::parse_u64_from_json_object(squad_json_object, "count")?;

        let squad = Squad::new(squad_id, owner, Position(x, y), count);
        squads.insert(squad_id, squad);
    }

    Ok(squads)
}
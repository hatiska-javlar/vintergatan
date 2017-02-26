use std::collections::HashMap;

use rustc_serialize::json::{Json, Object};

use common::{Id, PlayerId, ParseCommandResult, Position, utils};
use server::planet::Planet;
use server::player::Player;
use server::squad::Squad;

type Result<T> = ParseCommandResult<T>;

pub fn parse_command(string: &str) -> Result<(String, Object)> {
    let json = utils::parse_json(string)?;
    let params = utils::parse_json_as_object(&json)?;

    let command = utils::parse_string_from_json_object(params, "action")?;
    let data = utils::parse_object_from_json_object(params, "data")?;

    return Ok((command.to_string(), data.to_owned()));
}

pub fn parse_squad_spawn_command_data(data: &Object) -> Result<Id> {
    utils::parse_id_from_json_object(data, "planet_id")
}

pub fn parse_squad_move_command_data(data: &Object) -> Result<(Id, f64, f64)> {
    let squad_id = utils::parse_id_from_json_object(data, "squad_id")?;
    let x = utils::parse_f64_from_json_object(data, "x")?;
    let y = utils::parse_f64_from_json_object(data, "y")?;

    return Ok((squad_id, x, y));
}

pub fn format_process_command(
    player: &Player,
    planets_json: &String,
    players_json: &String,
    squads_json: &String
) -> String {
    format!(
        r#"{{"planets":{},"players":{},"squads":{},"id":{},"gold":{}}}"#,
        planets_json,
        players_json,
        squads_json,
        player.id(),
        player.gold()
    )
}

pub fn format_planets(planets: &HashMap<Id, Planet>) -> String {
    let formatted_planets = planets
        .values()
        .map(|planet| {
            let Position(x, y) = planet.position();
            let owner = planet.owner().map_or("null".to_string(), |owner| owner.to_string());

            format!(
                r#"{{"id":{},"x":{},"y":{},"owner":{}}}"#,
                planet.id(),
                x,
                y,
                owner
            )
        })
        .collect::<Vec<String>>();

    format!("[{}]", utils::join(formatted_planets, ","))
}

pub fn format_players(players: &HashMap<PlayerId, Player>) -> String {
    let formatted_players = players
        .values()
        .map(|player| format!(r#"{{"id":{}}}"#, player.id()))
        .collect::<Vec<String>>();

    format!("[{}]", utils::join(formatted_players, ","))
}

pub fn format_squads(squads: &HashMap<Id, Squad>) -> String {
    let formatted_squads = squads
        .values()
        .map(|squad| {
            let Position(x, y) = squad.position();

            format!(
                r#"{{"id":{},"owner":{},"x":{},"y":{},"count":{}}}"#,
                squad.id(),
                squad.owner(),
                x,
                y,
                squad.count()
            )
        })
        .collect::<Vec<String>>();

    format!("[{}]", utils::join(formatted_squads, ","))
}
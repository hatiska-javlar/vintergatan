use std::collections::HashMap;

use rustc_serialize::json::{Json, Object};

use client::player::Player;
use client::squad::Squad;
use client::waypoint::{Waypoint, WaypointType};
use common::{Id, PlayerId, ParseCommandError, ParseCommandResult, Position};
use common::utils::json;

type Result<T> = ParseCommandResult<T>;

type ProcessCommandTuple = (
    HashMap<Id, Waypoint>,
    HashMap<PlayerId, Player>,
    HashMap<Id, Squad>,
    PlayerId,
    f64
);

pub fn parse_process_command(string: &str) -> Result<ProcessCommandTuple> {
    let json = json::parse_json(string)?;
    let params = json::parse_json_as_object(&json)?;

    let process_command_tuple = (
        parse_waypoints(params)?,
        parse_players(params)?,
        parse_squads(params)?,
        json::parse_player_id_from_json_object(params, "id")?,
        json::parse_f64_from_json_object(params, "gold")?
    );

    return Ok(process_command_tuple);
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

fn parse_waypoints(params: &Object) -> Result<HashMap<Id, Waypoint>> {
    let waypoints_json_array = json::parse_array_from_json_object(params, "waypoints")?;

    let mut waypoints = HashMap::new();
    for waypoint_json in waypoints_json_array.into_iter() {
        let waypoint_json_object = json::parse_json_as_object(waypoint_json)?;

        let waypoint_id = json::parse_id_from_json_object(waypoint_json_object, "id")?;
        let x = json::parse_f64_from_json_object(waypoint_json_object, "x")?;
        let y = json::parse_f64_from_json_object(waypoint_json_object, "y")?;

        let owner = json::parse_option_player_id_from_json_object(waypoint_json_object, "owner")?;

        let waypoint_type = match json::parse_string_from_json_object(waypoint_json_object, "type")? {
            "asteroid" => WaypointType::Asteroid,
            "black_hole" => WaypointType::BlackHole,
            "planetoid" => WaypointType::Planetoid,
            "planet" => WaypointType::Planet,
            _ => unreachable!()
        };

        let waypoint = Waypoint::new(waypoint_id, waypoint_type,Position(x, y), owner);
        waypoints.insert(waypoint_id, waypoint);
    }

    Ok(waypoints)
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
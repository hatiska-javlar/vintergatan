use std::collections::HashMap;

use rustc_serialize::json::{Json, Object};

use common::{Id, PlayerId, ParseCommandResult, Position, utils};
use common::utils::json;
use server::player::{Player, PlayerState};
use server::squad::Squad;
use server::waypoint::{Waypoint, WaypointType};

type Result<T> = ParseCommandResult<T>;

pub fn parse_command(string: &str) -> Result<(String, Object)> {
    let json = json::parse_json(string)?;
    let params = json::parse_json_as_object(&json)?;

    let command = json::parse_string_from_json_object(params, "action")?;
    let data = json::parse_object_from_json_object(params, "data")?;

    return Ok((command.to_string(), data.to_owned()));
}

pub fn parse_squad_spawn_command_data(data: &Object) -> Result<Id> {
    json::parse_id_from_json_object(data, "planet_id")
}

pub fn parse_squad_move_command_data(data: &Object) -> Result<(Id, Id)> {
    let squad_id = json::parse_id_from_json_object(data, "squad_id")?;
    let waypoint_id = json::parse_id_from_json_object(data, "waypoint_id")?;

    return Ok((squad_id, waypoint_id));
}

pub fn format_process_command(
    player: &Player,
    waypoints_json: &String,
    players_json: &String,
    squads_json: &String
) -> String {
    format!(
        r#"{{"waypoints":{},"players":{},"squads":{},"id":{},"gold":{}}}"#,
        waypoints_json,
        players_json,
        squads_json,
        player.id(),
        player.gold()
    )
}

pub fn format_waypoints(waypoints: &HashMap<Id, Waypoint>) -> String {
    let formatted_waypoints = waypoints
        .values()
        .map(|waypoint| {
            let Position(x, y) = waypoint.position();
            let owner = waypoint.owner().map_or("null".to_string(), |owner| owner.to_string());

            format!(
                r#"{{"id":{},"x":{},"y":{},"owner":{},"type":"{}"}}"#,
                waypoint.id(),
                x,
                y,
                owner,
                match waypoint.waypoint_type() {
                    WaypointType::Planetoid => "planetoid",
                    WaypointType::Asteroid => "asteroid",
                    WaypointType::Planet => "planet",
                    WaypointType::BlackHole => "black_hole"
                }
            )
        })
        .collect::<Vec<String>>();

    format!("[{}]", utils::join(formatted_waypoints, ","))
}

pub fn format_players(players: &HashMap<PlayerId, Player>) -> String {
    let formatted_players = players
        .values()
        .map(|player| {
            let player_state = format_player_state(&player);

            format!(
                r#"{{"id":{},"name":"{}","state":"{}"}}"#,
                player.id(),
                player.name(),
                player_state
            )
        })
        .collect::<Vec<String>>();

    format!("[{}]", utils::join(formatted_players, ","))
}

fn format_player_state(player: &Player) -> String {
    let state = match *player.state() {
        PlayerState::Pending => "pending",
        PlayerState::Playing => "playing",
        PlayerState::Ready => "ready",
        PlayerState::Win => "win",
        PlayerState::Loose => "loose"
    };

    state.to_string()
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
                squad.life().ceil()
            )
        })
        .collect::<Vec<String>>();

    format!("[{}]", utils::join(formatted_squads, ","))
}
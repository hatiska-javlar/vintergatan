use std::collections::HashMap;

use rustc_serialize::json::{
    Json,
    Object
};
use ws::{
    Message,
    Sender
};

use common::PlayerId;
use common::id::Id;
use common::position::Position;
use common::to_command::ToCommand;
use client::planet::Planet;
use client::player::Player;
use client::squad::Squad;

pub enum Command {
    Connect {
        sender: Sender
    },

    Process {
        sender: Sender,
        planets: HashMap<Id, Planet>,
        players: HashMap<PlayerId, Player>,
        gold: f64,
        me: PlayerId
    },

    Disconnect {
        sender: Sender
    }
}

impl ToCommand for Command {
    fn connect(sender: Sender) -> Self {
        Command::Connect { sender: sender }
    }

    fn process(sender: Sender, message: Message) -> Result<Self, ()> {
        let raw = message.into_text().unwrap_or("".to_string());
        let parsed = Json::from_str(&raw).unwrap_or(Json::Object(Object::new()));

        let empty_json_object = Object::new();
        let params = parsed.as_object().unwrap_or(&empty_json_object);

        let process_command = Command::Process {
            sender: sender,
            planets: Self::parse_planets_from_json(params.get("planets")),
            players: Self::parse_players_from_json(params.get("players")),
            gold: params.get("gold").unwrap().as_f64().unwrap(),
            me: params.get("id").unwrap().as_f64().unwrap() as PlayerId
        };

        Ok(process_command)
    }

    fn disconnect(sender: Sender) -> Self {
        Command::Disconnect { sender: sender }
    }
}

impl Command {
    fn parse_planets_from_json(planets_json: Option<&Json>) -> HashMap<Id, Planet> {
        let planets_json_array = planets_json.unwrap().as_array().unwrap();

        let mut planets = HashMap::new();
        for planet_json in planets_json_array.into_iter() {
            let planet_json_object = planet_json.as_object().unwrap();

            let id = planet_json_object.get("id").unwrap().as_u64().unwrap();
            let x = planet_json_object.get("x").unwrap().as_f64().unwrap();
            let y = planet_json_object.get("y").unwrap().as_f64().unwrap();
            let position = Position(x, y);
            let owner = planet_json_object.get("owner").unwrap().as_u64().map(|owner| owner as PlayerId);

            let planet = Planet::new(id, position, owner);
            planets.insert(id, planet);
        }

        planets
    }

    fn parse_players_from_json(players_json: Option<&Json>) -> HashMap<PlayerId, Player> {
        let players_json_array = players_json.unwrap().as_array().unwrap();

        let mut players = HashMap::new();
        for player_json in players_json_array.into_iter() {
            let player_json_object = player_json.as_object().unwrap();

            let id = player_json_object.get("id").unwrap().as_u64().unwrap() as PlayerId;
            let squads = Self::parse_squads_from_json(player_json_object.get("squads"));

            let player = Player::new(id, squads);
            players.insert(id, player);
        }

        players
    }

    fn parse_squads_from_json(squads_json: Option<&Json>) -> HashMap<Id, Squad> {
        let squads_json_array = squads_json.unwrap().as_array().unwrap();

        let mut squads = HashMap::new();
        for squad_json in squads_json_array.into_iter() {
            let squad_json_object = squad_json.as_object().unwrap();

            let id = squad_json_object.get("id").unwrap().as_u64().unwrap() as Id;
            let x = squad_json_object.get("x").unwrap().as_f64().unwrap();
            let y = squad_json_object.get("y").unwrap().as_f64().unwrap();
            let count = squad_json_object.get("count").unwrap().as_u64().unwrap();

            let squad = Squad::new(id, Position(x, y), count);
            squads.insert(id, squad);
        }

        squads
    }
}

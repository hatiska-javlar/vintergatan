use rustc_serialize::json::{
    Json,
    Object
};
use ws::{
    Message,
    Sender
};

use common::position::Position;
use common::to_command::ToCommand;
use client::data::PlanetData;
use client::planet::Planet;

pub enum Command {
    Connect {
        sender: Sender
    },

    Process {
        sender: Sender,
        planets: Vec<PlanetData>
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
            planets: Self::parse_planets_from_json(params.get("planets"))
        };

        Ok(process_command)
    }

    fn disconnect(sender: Sender) -> Self {
        Command::Disconnect { sender: sender }
    }
}

impl Command {
    fn parse_planets_from_json(planets_json: Option<&Json>) -> Vec<PlanetData> {
        planets_json
            .unwrap()
            .as_array()
            .unwrap()
            .into_iter()
            .map(|planet_json| {
                let planet_json_object = planet_json.as_object().unwrap();

                let id = planet_json_object.get("id").unwrap().as_u64().unwrap();
                let x = planet_json_object.get("x").unwrap().as_f64().unwrap();
                let y = planet_json_object.get("y").unwrap().as_f64().unwrap();
                let owner = planet_json_object.get("owner").unwrap().as_u64().map(|owner| owner as usize);

                PlanetData {
                    id: id,
                    position: Position(x, y),
                    owner: owner
                }
            })
            .collect()
    }
}
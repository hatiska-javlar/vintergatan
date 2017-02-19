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
use client::planet::Planet;

pub enum Command {
    Connect {
        sender: Sender
    },

    Process {
        sender: Sender,
        planets: Vec<Planet>
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

        if let Some(planets_json) = params.get("planets") {
            let planets = planets_json.as_array().unwrap().into_iter().map(|planet_json| {
                let id = planet_json.as_object().unwrap().get("id").unwrap().as_u64().unwrap();
                let x = planet_json.as_object().unwrap().get("x").unwrap().as_f64().unwrap();
                let y = planet_json.as_object().unwrap().get("y").unwrap().as_f64().unwrap();

                Planet::new(id, Position(x, y))
            }).collect();

            let process_command = Command::Process {
                sender: sender,
                planets: planets
            };

            return Ok(process_command);
        }

        Err(())
    }

    fn disconnect(sender: Sender) -> Self {
        Command::Disconnect { sender: sender }
    }
}
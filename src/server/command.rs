use rustc_serialize::json::{Json, Object};
use ws::{
    Message,
    Sender
};

use common::to_command::ToCommand;
use common::id::Id;

pub enum Command {
    Connect {
        sender: Sender
    },

    Process {
        sender: Sender
    },

    Spawn {
        sender: Sender,
        planet_id: Id
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

        if let Some(planet_id) = params.get("spawn") {
            return Ok(Command::Spawn { sender: sender, planet_id: planet_id.as_u64().unwrap() });
        }

        Ok(Command::Process { sender: sender })
    }

    fn disconnect(sender: Sender) -> Self {
        Command::Disconnect { sender: sender }
    }
}
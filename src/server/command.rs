use ws::{Message, Sender};

use common::to_command::ToCommand;
use common::{Id, ParseCommandError, ParseCommandResult};
use server::json;

pub enum Command {
    Connect {
        sender: Sender
    },

    Ready {
        sender: Sender
    },

    SquadSpawn {
        sender: Sender,
        planet_id: Id
    },

    SquadMove {
        sender: Sender,
        squad_id: Id,
        waypoint_id: Id
    },

    Disconnect {
        sender: Sender
    }
}

impl ToCommand for Command {
    fn connect(sender: Sender) -> Self {
        Command::Connect { sender: sender }
    }

    fn process(sender: Sender, message: &Message) -> ParseCommandResult<Self> {
        let raw = message.as_text()
            .map_err(ParseCommandError::BrokenCommand)?;

        let (action, data) = json::parse_command(raw)?;

        let command = match action.as_ref() {
            "ready" => {
                Command::Ready { sender: sender }
            },

            "squad_spawn" => {
                let planet_id = json::parse_squad_spawn_command_data(&data)?;

                Command::SquadSpawn {
                    sender: sender,
                    planet_id: planet_id
                }
            },

            "squad_move" => {
                let (squad_id, waypoint_id) = json::parse_squad_move_command_data(&data)?;

                Command::SquadMove {
                    sender,
                    squad_id,
                    waypoint_id
                }
            },

            _ => return Err(ParseCommandError::UnsupportedAction)
        };

        Ok(command)
    }

    fn disconnect(sender: Sender) -> Self {
        Command::Disconnect { sender: sender }
    }
}

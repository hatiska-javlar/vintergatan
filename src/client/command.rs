use std::collections::HashMap;

use ws::{Message, Sender};

use common::{Id, ParseCommandError, ParseCommandResult, PlayerId};
use common::to_command::ToCommand;
use client::json;
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
        squads: HashMap<Id, Squad>,
        gold: f64,
        me: PlayerId
    },

    Disconnect {
        sender: Sender
    },

    SetState {
        state: String
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
            "set_state" => {
                let state = json::parse_state_data(&data)?;

                Command::SetState { state: state }
            },

            "render" => {
                let (planets, players, squads, me, gold) = json::parse_process_command_data(&data)?;

                Command::Process {
                    sender: sender,
                    planets: planets,
                    players: players,
                    squads: squads,
                    gold: gold,
                    me: me
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
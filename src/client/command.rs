use std::collections::HashMap;

use ws::{Message, Sender};

use common::{Id, ParseCommandError, ParseCommandResult, PlayerId};
use common::to_command::ToCommand;
use client::json;
use client::player::Player;
use client::squad::Squad;
use client::waypoint::Waypoint;

pub enum Command {
    Connect {
        sender: Sender
    },

    Process {
        sender: Sender,
        waypoints: HashMap<Id, Waypoint>,
        players: HashMap<PlayerId, Player>,
        squads: HashMap<Id, Squad>,
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

    fn process(sender: Sender, message: &Message) -> ParseCommandResult<Self> {
        let raw = message.as_text()
            .map_err(ParseCommandError::BrokenCommand)?;

        let (waypoints, players, squads, me, gold) = json::parse_process_command(raw)?;

        let command = Command::Process {
            sender: sender,
            waypoints: waypoints,
            players: players,
            squads: squads,
            gold: gold,
            me: me
        };

        Ok(command)
    }

    fn disconnect(sender: Sender) -> Self {
        Command::Disconnect { sender: sender }
    }
}
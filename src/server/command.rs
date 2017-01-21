use ws::{
    Message,
    Sender
};

use common::to_command::ToCommand;

pub enum Command {
    Connect {
        sender: Sender
    },

    Process {
        sender: Sender
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
        Ok(Command::Process { sender: sender })
    }

    fn disconnect(sender: Sender) -> Self {
        Command::Disconnect { sender: sender }
    }
}
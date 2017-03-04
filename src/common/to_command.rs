use ws::{Sender, Message};

use common::ParseCommandResult;

pub trait ToCommand {
    fn connect(sender: Sender) -> Self;

    fn process(sender: Sender, message: &Message)
        -> ParseCommandResult<Self>
        where Self: Sized;

    fn disconnect(sender: Sender) -> Self;
}
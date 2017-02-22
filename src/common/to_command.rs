use ws::{
    Sender,
    Message
};

pub trait ToCommand {
    fn connect(sender: Sender) -> Self;
    fn process(sender: Sender, message: Message) -> Result<Self, ()> where Self: Sized;
    fn disconnect(sender: Sender) -> Self;
}
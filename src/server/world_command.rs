use ws::Sender;

pub enum WorldCommand {
    Connect {
        sender: Sender
    }
}

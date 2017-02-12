use ws::Sender;

pub type PlayerId = usize;

pub struct Player {
    sender: Sender
}

impl Player {
    pub fn new(sender: Sender) -> Player {
        Player { sender: sender }
    }

    pub fn id(&self) -> PlayerId {
        self.sender.token().as_usize()
    }

    pub fn send(&self, message: String) {
        self.sender.send(message);
    }
}
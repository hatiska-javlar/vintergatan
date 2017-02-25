use ws::Sender;

use common::PlayerId;

pub struct Player {
    sender: Sender,
    gold: f64
}

impl Player {
    pub fn new(sender: Sender) -> Player {
        Player {
            sender: sender,
            gold: 0.0
        }
    }

    pub fn id(&self) -> PlayerId {
        self.sender.token().as_usize()
    }

    pub fn send(&self, message: String) {
        self.sender.send(message);
    }

    pub fn gold(&self) -> f64 {
        self.gold
    }

    pub fn set_gold(&mut self, gold: f64) {
        self.gold = gold;
    }
}
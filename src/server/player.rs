use ws::Sender;

use common::PlayerId;

pub struct Player {
    sender: Sender,
    state: PlayerState,
    name: String,
    gold: f64
}

pub enum PlayerState {
    Pending,
    Ready,
    Playing,
    Win,
    Loose
}

impl Player {
    pub fn new(sender: Sender, name: String) -> Player {
        Player {
            sender: sender,
            state: PlayerState::Pending,
            name: name,
            gold: 0.0
        }
    }

    pub fn id(&self) -> PlayerId {
        self.sender.token().as_usize()
    }

    pub fn send(&self, message: String) {
        self.sender.send(message);
    }

    pub fn state(&self) -> &PlayerState {
        &self.state
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn gold(&self) -> f64 {
        self.gold
    }

    pub fn set_gold(&mut self, gold: f64) {
        self.gold = gold;
    }

    pub fn is_pending(&self) -> bool {
        match self.state {
            PlayerState::Pending => true,
            _ => false
        }
    }

    pub fn is_ready(&self) -> bool {
        match self.state {
            PlayerState::Ready => true,
            _ => false
        }
    }

    pub fn is_playing(&self) -> bool {
        match self.state {
            PlayerState::Playing => true,
            _ => false
        }
    }

    pub fn is_win(&self) -> bool {
        match self.state {
            PlayerState::Win => true,
            _ => false
        }
    }

    pub fn set_ready_state(&mut self) {
        if self.is_pending() {
            self.state = PlayerState::Ready;
        }
    }

    pub fn set_playing_state(&mut self) {
        if self.is_ready() {
            self.state = PlayerState::Playing;
        }
    }

    pub fn set_win_state(&mut self) {
        if self.is_playing() {
            self.state = PlayerState::Win;
        }
    }

    pub fn set_loose_state(&mut self) {
        if self.is_playing() {
            self.state = PlayerState::Loose;
        }
    }
}
use std::collections::HashMap;
use std::collections::hash_map::Values;

use ws::Sender;

use common::id::Id;
use common::position::Position;
use server::squad::Squad;

pub type PlayerId = usize;

pub struct Player {
    sender: Sender,
    squads: HashMap<Id, Squad>
}

impl Player {
    pub fn new(sender: Sender) -> Player {
        Player {
            sender: sender,
            squads: HashMap::new()
        }
    }

    pub fn id(&self) -> PlayerId {
        self.sender.token().as_usize()
    }

    pub fn send(&self, message: String) {
        self.sender.send(message);
    }

    pub fn squads(&self) -> Values<Id, Squad> {
        self.squads.values()
    }

    pub fn add_squad(&mut self, id: Id, position: Position) {
        let squad = Squad::new(id, position);
        self.squads.insert(squad.id(), squad);
    }

    pub fn move_squad(&mut self, id: Id, position: Position) {
        if let Some(squad) = self.squads.get_mut(&id) {
            squad.set_position(position);
        }
    }
}
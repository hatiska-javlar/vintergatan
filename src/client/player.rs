use std::collections::HashMap;

use common::id::Id;
use common::PlayerId;
use client::squad::Squad;

pub struct Player {
    id: PlayerId,
    squads: HashMap<Id, Squad>
}

impl Player {
    pub fn new(id: PlayerId, squads: HashMap<Id, Squad>) -> Player {
        Player {
            id: id,
            squads: squads
        }
    }

    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn squads(&self) -> &HashMap<Id, Squad> {
        &self.squads
    }

    pub fn squads_mut(&mut self) -> &mut HashMap<Id, Squad> {
        &mut self.squads
    }
}
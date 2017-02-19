use common::id::Id;
use common::position::Position;
use server::player::PlayerId;

pub struct Planet {
    id: Id,
    owner: Option<PlayerId>,
    position: Position
}

impl Planet {
    pub fn new(id: Id, position: Position) -> Planet {
        Planet {
            id: id,
            position: position,
            owner: None
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn owner(&self) -> Option<PlayerId> {
        self.owner
    }

    pub fn set_owner(&mut self, owner: Option<PlayerId>) {
        self.owner = owner;
    }
}
use common::id::Id;
use common::position::Position;
use common::PlayerId;

pub struct Planet {
    id: Id,
    position: Position,
    owner: Option<PlayerId>
}

impl Planet {
    pub fn new(id: Id, position: Position, owner: Option<PlayerId>) -> Planet {
        Planet {
            id: id,
            position: position,
            owner: owner
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

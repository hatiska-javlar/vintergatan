use common::id::Id;
use common::PlayerId;
use common::position::Position;

pub struct Squad {
    id: Id,
    owner: PlayerId,
    position: Position,
    count: u64
}

impl Squad {
    pub fn new(id: Id, owner: PlayerId, position: Position, count: u64) -> Squad {
        Squad {
            id: id,
            owner: owner,
            position: position,
            count: count
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn owner(&self) -> PlayerId {
        self.owner
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn set_count(&mut self, count: u64) {
        self.count = count;
    }
}
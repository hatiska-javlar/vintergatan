use common::id::Id;
use common::position::Position;

pub struct Squad {
    id: Id,
    position: Position,
    count: u64
}

impl Squad {
    pub fn new(id: Id, position: Position) -> Squad {
        Squad {
            id: id,
            position: position,
            count: 10
        }
    }

    pub fn id(&self) -> Id {
        self.id
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
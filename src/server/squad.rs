use common::id::Id;
use common::position::Position;

pub struct Squad {
    id: Id,
    state: SquadState,
    position: Position,
    count: u64
}

#[derive(Copy, Clone)]
pub enum SquadState {
    Pending,
    Moving {
        destination: Position
    }
}

impl Squad {
    pub fn new(id: Id, position: Position) -> Squad {
        Squad {
            id: id,
            state: SquadState::Pending,
            position: position,
            count: 10
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn state(&self) -> SquadState {
        self.state
    }

    pub fn set_state(&mut self, state: SquadState) {
        self.state = state;
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

    pub fn move_to(&mut self, position: Position) {
        self.state = SquadState::Moving { destination: position };
    }
}
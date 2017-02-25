use common::{Id, PlayerId, Position};

pub struct Squad {
    id: Id,
    owner: PlayerId,
    state: SquadState,
    position: Position,
    count: u64
}

#[derive(Copy, Clone)]
pub enum SquadState {
    Pending,
    Moving {
        destination: Position
    },
    OnOrbit {
        planet_id: Id
    }
}

impl Squad {
    pub fn new(id: Id, owner: PlayerId, position: Position) -> Squad {
        Squad {
            id: id,
            owner: owner,
            state: SquadState::Pending,
            position: position,
            count: 10
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn owner(&self) -> PlayerId {
        self.owner
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

    pub fn is_on_orbit(&self, orbit_planet_id: Id) -> bool {
        match self.state {
            SquadState::OnOrbit { planet_id } => planet_id == orbit_planet_id,
            _ => false
        }
    }
}
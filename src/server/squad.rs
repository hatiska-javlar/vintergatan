use common::{Id, PlayerId, Position};

pub struct Squad {
    id: Id,
    owner: PlayerId,
    state: SquadState,
    position: Position,
    life: f64
}

#[derive(Copy, Clone)]
pub enum SquadState {
    InSpace,
    Moving {
        destination: Position
    },
    OnOrbit {
        waypoint_id: Id
    }
}

impl Squad {
    pub fn new(id: Id, owner: PlayerId, position: Position) -> Squad {
        Squad {
            id: id,
            owner: owner,
            state: SquadState::InSpace,
            position: position,
            life: 10_f64
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

    pub fn life(&self) -> f64 {
        self.life
    }

    pub fn set_life(&mut self, life: f64) {
        self.life = life;
    }

    pub fn move_to(&mut self, position: Position) {
        self.state = SquadState::Moving { destination: position };
    }

    pub fn is_on_orbit(&self, orbit_waypoint_id: Id) -> bool {
        match self.state {
            SquadState::OnOrbit { waypoint_id } => waypoint_id == orbit_waypoint_id,
            _ => false
        }
    }

    pub fn is_standing(&self) -> bool {
        match self.state {
            SquadState::InSpace | SquadState::OnOrbit { .. } => true,
            SquadState::Moving { .. } => false
        }
    }
}
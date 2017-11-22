use common::{Id, PlayerId, Position};

#[derive(PartialEq, Copy, Clone)]
pub enum WaypointType {
    Planet,
    Planetoid,
    Asteroid,
    BlackHole
}

pub struct Waypoint {
    id: Id,
    waypoint_type: WaypointType,
    owner: Option<PlayerId>,
    position: Position
}

impl Waypoint {
    pub fn new(id: Id, waypoint_type: WaypointType, position: Position) -> Waypoint {
        Waypoint {
            id,
            waypoint_type,
            position,
            owner: None
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn waypoint_type(&self) -> WaypointType {
        self.waypoint_type
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
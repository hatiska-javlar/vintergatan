use std::collections::HashMap;

use common::id::Id;
use common::position::Position;
use common::PlayerId;

pub struct PlanetData {
    pub id: Id,
    pub position: Position,
    pub owner: Option<PlayerId>
}

pub struct PlayerData {
    pub id: PlayerId,
    pub squads_data: HashMap<Id, SquadData>
}

pub struct SquadData {
    pub id: Id,
    pub position: Position,
    pub count: u64
}
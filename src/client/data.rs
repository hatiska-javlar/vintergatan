use common::id::Id;
use common::position::Position;
use common::PlayerId;

pub struct PlanetData {
    pub id: Id,
    pub position: Position,
    pub owner: Option<PlayerId>
}
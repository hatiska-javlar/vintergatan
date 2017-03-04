use common::PlayerId;

pub struct Player {
    id: PlayerId
}

impl Player {
    pub fn new(id: PlayerId) -> Player {
        Player {
            id: id
        }
    }

    pub fn id(&self) -> PlayerId {
        self.id
    }
}
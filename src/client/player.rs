use common::PlayerId;

pub struct Player {
    id: PlayerId,
    name: String,
    state: String
}

impl Player {
    pub fn new(id: PlayerId, name: String, state: String) -> Player {
        Player {
            id: id,
            name: name,
            state: state
        }
    }

    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn state(&self) -> &String {
        &self.state
    }
}
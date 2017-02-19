use common::id::Id;
use common::position::Position;

pub struct Planet {
    id: Id,
    position: Position,
    color: [f32; 4],
    size: f64
}

impl Planet {
    pub fn new(id: Id, position: Position) -> Planet {
        Planet {
            id: id,
            position: position,
            color: [0.125490196, 0.752941176, 0.870588235, 1.0],
            size: 10f64
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn color(&self) -> [f32; 4] {
        self.color
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn size(&self) -> f64 {
        self.size
    }
}
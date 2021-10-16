use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Command {
    id: u32,
    point: (f32, f32),
    angle: f32,
    length: f32,
}

impl Command {
    pub fn new(id: u32, point: (f32, f32), angle: f32, length: f32) -> Self {
        Self {
            id,
            point,
            angle,
            length,
        }
    }

    /// Monotonically incremental ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// in x, y
    pub fn point(&self) -> (f32, f32) {
        self.point
    }

    /// In range -Pi -> Pi
    pub fn angle(&self) -> f32 {
        self.angle
    }

    pub fn length(&self) -> f32 {
        self.length
    }
}

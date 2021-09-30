use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PoseClientUpdate {
    objects: Vec<ObjectPose>,
    delete: Vec<String>,
}

impl PoseClientUpdate {
    pub fn new() -> Self {
        PoseClientUpdate {
            objects: vec![],
            delete: vec![],
        }
    }

    pub fn add(&mut self, id: &str, pose: (f32, f32, f32)) -> &mut ObjectPose {
        let pose = ObjectPose::new(id, pose);
        self.objects.push(pose);
        let index = self.objects.len() - 1;
        self.objects.get_mut(index).unwrap()
    }

    pub fn delete(&mut self, id: &str) {
        self.delete.push(id.to_owned());
    }

    pub fn updates(&self) -> &Vec<ObjectPose> {
        &self.objects
    }

    pub fn deletions(&self) -> &Vec<String> {
        &self.delete
    }
}

/// in form (x, y, z, w)
type Quaternion = (f32, f32, f32, f32);

const DEFAULT_TIMEOUT: f32 = 5.;
const DEFAULT_RED_COLOR: Color = Color::Red;
const DEFAULT_SHAPE: Shape = Shape::Sphere(0.05);
const IDENTITY_QUATERNION: Quaternion = (0., 0., 0., 1.);

#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectPose {
    pub id: String,
    pub pose: (f32, f32, f32),
    pub timeout: f32,
    pub shape: Shape,
    pub color: Color,
    pub rotation: Quaternion,
}

impl ObjectPose {
    fn new(id: &str, pose: (f32, f32, f32)) -> Self {
        ObjectPose {
            id: id.to_owned(),
            pose,
            timeout: DEFAULT_TIMEOUT,
            shape: DEFAULT_SHAPE,
            color: DEFAULT_RED_COLOR,
            rotation: IDENTITY_QUATERNION,
        }
    }

    pub fn with_timeout(&mut self, timeout: f32) -> &mut Self {
        self.timeout = timeout;
        self
    }

    pub fn with_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    pub fn with_shape(&mut self, shape: Shape) -> &mut Self {
        self.shape = shape;
        self
    }

    /// in form (x, y, z, w)
    pub fn with_rotation(&mut self, rotation: Quaternion) -> &mut Self {
        self.rotation = rotation;
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Sphere(f32),
    Cube(f32, f32, f32),
    Line((f32, f32, f32)),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Red,
    Green,
    Blue,
    Cyan,
    Magenta,
    Yellow,
}

impl Color {
    pub fn to_rgb(&self) -> (f32, f32, f32) {
        match self {
            Color::Red => (1., 0., 0.),
            Color::Green => (0., 1., 0.),
            Color::Blue => (0., 0., 1.),
            Color::Cyan => (0., 1., 1.),
            Color::Magenta => (1., 0., 1.),
            Color::Yellow => (1., 1., 0.),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Blue => "Blue",
            Color::Cyan => "Cyan",
            Color::Magenta => "Magenta",
            Color::Yellow => "Yellow",
        }
    }
}

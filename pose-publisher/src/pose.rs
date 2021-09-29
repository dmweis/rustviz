use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PoseClientUpdate {
    client_id: String,
    objects: Vec<ObjectPose>,
    delete: Vec<String>,
}

impl PoseClientUpdate {
    pub fn new(client_id: &str) -> Self {
        PoseClientUpdate {
            client_id: client_id.to_owned(),
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

const DEFAULT_TIMEOUT: f32 = 5.;
const DEFAULT_RED_COLOR: Color = Color::Red;
const DEFAULT_SHAPE: Shape = Shape::Sphere(0.05);

#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectPose {
    pub id: String,
    pub pose: (f32, f32, f32),
    pub timeout: f32,
    pub shape: Shape,
    pub color: Color,
}

impl ObjectPose {
    fn new(id: &str, pose: (f32, f32, f32)) -> Self {
        ObjectPose {
            id: id.to_owned(),
            pose,
            timeout: DEFAULT_TIMEOUT,
            shape: DEFAULT_SHAPE,
            color: DEFAULT_RED_COLOR,
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
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Sphere(f32),
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
}

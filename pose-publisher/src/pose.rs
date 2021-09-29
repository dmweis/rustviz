use nalgebra as na;
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

    pub fn add(&mut self, object: ObjectPose) {
        self.objects.push(object);
    }

    pub fn delete(&mut self, id: String) {
        self.delete.push(id);
    }

    pub fn updates(&self) -> &Vec<ObjectPose> {
        &self.objects
    }

    pub fn deletions(&self) -> &Vec<String> {
        &self.delete
    }
}

const DEFAULT_TIMEOUT: f32 = 5.;
const DEFAULT_RED_COLOR: na::Vector3<f32> = na::Vector3::new(1., 0., 0.);
const DEFAULT_SHAPE: Shape = Shape::Sphere;

#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectPose {
    pub id: String,
    pub pose: na::Point3<f32>,
    pub timeout: f32,
    pub shape: Shape,
    pub color: na::Vector3<f32>,
}

impl ObjectPose {
    pub fn with_defaults(id: String, pose: na::Point3<f32>) -> Self {
        ObjectPose {
            id,
            pose,
            timeout: DEFAULT_TIMEOUT,
            shape: DEFAULT_SHAPE,
            color: DEFAULT_RED_COLOR,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Sphere,
}

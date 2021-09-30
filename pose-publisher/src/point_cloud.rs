use crate::pose::Color;
use serde::{Deserialize, Serialize};

const DEFAULT_TIMEOUT: f32 = 5.;
const DEFAULT_RED_COLOR: Color = Color::Red;

type Point2 = (f32, f32);

#[derive(Serialize, Deserialize, Debug)]
pub struct PointCloud2 {
    id: String,
    parent_frame_id: Option<String>,
    points: Vec<Point2>,
    timeout: f32,
    color: Color,
}

impl PointCloud2 {
    pub fn from_points(id: &str, points: Vec<Point2>) -> Self {
        Self {
            id: id.to_owned(),
            parent_frame_id: None,
            points,
            timeout: DEFAULT_TIMEOUT,
            color: DEFAULT_RED_COLOR,
        }
    }

    pub fn with_timeout(mut self, timeout: f32) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_parent_frame_id(mut self, frame_id: &str) -> Self {
        self.parent_frame_id = Some(frame_id.to_owned());
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn parent_frame_id(&self) -> &Option<String> {
        &self.parent_frame_id
    }

    pub fn points(&self) -> &Vec<Point2> {
        &self.points
    }

    pub fn color(&self) -> &Color {
        &self.color
    }

    pub fn timeout(&self) -> f32 {
        self.timeout
    }
}

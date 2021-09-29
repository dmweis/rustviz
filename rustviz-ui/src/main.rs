use anyhow::Result;
use clap::Clap;
use kiss3d::{light::Light, scene::SceneNode, window::Window};
use nalgebra as na;
use pose_publisher::{pose::Shape, ObjectPose, PoseSubscriber};
use std::{
    collections::HashMap,
    net::SocketAddrV4,
    time::{Duration, Instant},
};

fn convert_coordinate_system(position: na::Vector3<f32>) -> na::Vector3<f32> {
    na::Vector3::new(position.y, position.z, position.x)
}

fn attach_node_type(shape: Shape, window: &mut Window) -> SceneNode {
    match shape {
        Shape::Sphere => window.add_sphere(0.01),
    }
}

struct ObjectContainer {
    objects: HashMap<String, VisualizerObject>,
}

impl ObjectContainer {
    fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    fn update_object(&mut self, object: &ObjectPose, window: &mut Window) {
        if let Some(node_reference) = self.objects.get_mut(&object.id) {
            node_reference.update(object, window);
        } else {
            let node = VisualizerObject::new(object, window);
            self.objects.insert(object.id.clone(), node);
        }
    }

    fn delete_object(&mut self, id: &str) {
        if let Some(mut object) = self.objects.remove(id) {
            object.node.unlink();
        }
    }

    fn remove_timed_out(&mut self) {
        self.objects.retain(|_, node| !node.is_timed_out());
    }
}

struct VisualizerObject {
    node: SceneNode,
    current_shape: Shape,
    last_update: Instant,
    timeout: Duration,
}

impl VisualizerObject {
    fn new(object_info: &ObjectPose, window: &mut Window) -> Self {
        let scene_node = attach_node_type(object_info.shape, window);
        let mut object = Self {
            node: scene_node,
            timeout: Duration::from_secs_f32(object_info.timeout),
            last_update: Instant::now(),
            current_shape: object_info.shape,
        };
        object.update_pose(object_info.pose);
        object.update_color(object_info.color);
        object
    }

    fn update(&mut self, update: &ObjectPose, window: &mut Window) {
        self.touch();
        self.update_pose(update.pose);
        self.update_color(update.color);
        self.timeout = Duration::from_secs_f32(update.timeout);
        if self.current_shape != update.shape {
            self.node.unlink();
            self.node = attach_node_type(update.shape, window);
        }
    }

    fn update_pose(&mut self, pose: na::Point3<f32>) {
        self.node
            .set_local_translation(na::Translation3::from(convert_coordinate_system(
                pose.coords,
            )));
    }

    fn update_color(&mut self, color: na::Vector3<f32>) {
        self.node.set_color(color.x, color.y, color.z);
    }

    fn touch(&mut self) {
        self.last_update = Instant::now();
    }

    fn is_timed_out(&self) -> bool {
        self.last_update.elapsed() > self.timeout
    }
}

impl Drop for VisualizerObject {
    fn drop(&mut self) {
        self.node.unlink()
    }
}

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(short, long, default_value = "239.0.0.22:7072")]
    address: SocketAddrV4,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let pose_subscriber = PoseSubscriber::new(args.address).unwrap();
    let mut object_container = ObjectContainer::new();
    let mut window = Window::new("rustviz");

    window.set_background_color(0.5, 0.5, 0.5);
    window.set_light(Light::StickToCamera);
    add_ground_plane(&mut window);

    while window.render() {
        while let Ok(update) = pose_subscriber.next() {
            for object_update in update.updates() {
                object_container.update_object(object_update, &mut window);
            }
            for delete_id in update.deletions() {
                object_container.delete_object(delete_id);
            }
        }
        object_container.remove_timed_out();
    }
    Ok(())
}

fn add_ground_plane(window: &mut Window) {
    let size = 0.5;
    for i in 0..4 {
        for j in 0..4 {
            let mut cube = window.add_cube(size, size, 0.001);
            if (i + j) % 2 == 0 {
                // cube.set_color(1.0, 0.3, 0.2);
                cube.set_color(0.0, 0.0, 0.0);
            } else {
                // cube.set_color(0.5, 0.04, 0.17);
                cube.set_color(1.0, 1.0, 1.0);
            }
            let distance = (1_f32.powi(2) + 1_f32.powi(2)).sqrt();
            let x_ind = j as f32 - distance;
            let y_ind = i as f32 - distance;
            let trans = na::Isometry3::from_parts(
                na::Translation3::new(size * x_ind, 0.0, size * y_ind),
                na::UnitQuaternion::from_euler_angles(0.0, -1.57, -1.57),
            );
            cube.set_local_transformation(trans);
        }
    }
}

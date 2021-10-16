use anyhow::Result;
use clap::Clap;
use kiss3d::{
    camera::Camera,
    event::{Action, MouseButton, WindowEvent},
    light::Light,
    scene::SceneNode,
    window::Window,
};
use nalgebra as na;
use pose_publisher::{
    commands::Command,
    point_cloud::PointCloud2,
    pose::{Color, Shape},
    CommandPublisher, ObjectPose, PointCloudSubscriber, PoseSubscriber,
};
use std::{
    collections::HashMap,
    net::SocketAddrV4,
    time::{Duration, Instant},
};

fn convert_coordinate_system((x, y, z): (f32, f32, f32)) -> na::Vector3<f32> {
    na::Vector3::new(y, z, x)
}

fn inverse_convert_coordinate_system(vector: &na::Vector3<f32>) -> (f32, f32, f32) {
    (vector.z, vector.x, vector.y)
}

fn convert_rotation_coordinate_system(
    (x, y, z, w): (f32, f32, f32, f32),
) -> na::UnitQuaternion<f32> {
    na::UnitQuaternion::new_normalize(na::Quaternion::new(w, y, z, x))
}

fn attach_node_type(shape: Shape, window: &mut Window) -> Option<SceneNode> {
    match shape {
        Shape::Sphere(radius) => Some(window.add_sphere(radius)),
        Shape::Cube(x, y, z) => Some(window.add_cube(y, z, x)),
        Shape::Line(_) => None,
    }
}

struct ObjectContainer {
    objects: HashMap<String, VisualizerObject>,
    point_clouds: HashMap<String, PointCloudContainer>,
}

impl ObjectContainer {
    fn new() -> Self {
        Self {
            objects: HashMap::new(),
            point_clouds: HashMap::new(),
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
            if let Some(scene_node) = &mut object.node {
                scene_node.unlink();
            }
        }
    }

    fn update_point_clouds(&mut self, point_cloud: PointCloud2) {
        self.point_clouds.insert(
            point_cloud.id().to_owned(),
            PointCloudContainer::new(point_cloud),
        );
    }

    fn remove_timed_out(&mut self) {
        self.objects.retain(|_, node| !node.is_timed_out());
        self.point_clouds
            .retain(|_, point_cloud| !point_cloud.is_timed_out());
    }

    fn display_message(&self) -> String {
        let mut text_buffer = String::new();
        for (id, object) in &self.objects {
            text_buffer.push_str(&format!(
                "{}: {} [{:.2} {:.2} {:.2}] \n",
                id,
                object.last_color.name(),
                object.last_pose.0,
                object.last_pose.1,
                object.last_pose.2,
            ));
        }
        for (id, point_cloud) in &self.point_clouds {
            let parent_frame_id = point_cloud
                .point_cloud()
                .parent_frame_id()
                .clone()
                .unwrap_or_else(|| "N/A".to_owned());
            text_buffer.push_str(&format!(
                "{}: {} len {} \n",
                id,
                parent_frame_id,
                point_cloud.point_cloud().points().len()
            ));
        }
        text_buffer
    }

    fn draw_lines(&self, window: &mut Window) {
        for object in self.objects.values() {
            if let Shape::Line(end) = object.current_shape {
                let rgb = object.last_color.to_rgb();
                window.draw_line(
                    &convert_coordinate_system(object.last_pose).into(),
                    &convert_coordinate_system(end).into(),
                    &na::Point3::new(rgb.0, rgb.1, rgb.2),
                );
            }
        }
    }

    fn draw_point_clouds(&self, window: &mut Window) {
        for point_cloud in self.point_clouds.values() {
            let (root_point, root_rotation) =
                if let Some(parent_frame_id) = point_cloud.point_cloud().parent_frame_id() {
                    self.objects
                        .get(parent_frame_id)
                        .map(|node| (node.last_pose, node.last_rotation))
                        .unwrap_or_else(|| ((0., 0., 0.01), (0., 0., 0., 1.)))
                } else {
                    ((0., 0., 0.01), (0., 0., 0., 1.))
                };
            let rgb = point_cloud.point_cloud().color().to_rgb();
            let color = na::Point3::new(rgb.0, rgb.1, rgb.2);
            let root_translation = na::Isometry3::from_parts(
                na::Translation3::from(convert_coordinate_system(root_point)),
                convert_rotation_coordinate_system(root_rotation),
            );
            for point in point_cloud.point_cloud().points() {
                let point3 = root_translation
                    * na::Point3::from(convert_coordinate_system((point.0, point.1, 0.0)));
                window.draw_point(&point3, &color);
            }
        }
    }
}

struct VisualizerObject {
    node: Option<SceneNode>,
    current_shape: Shape,
    last_update: Instant,
    timeout: Duration,
    last_pose: (f32, f32, f32),
    last_rotation: (f32, f32, f32, f32),
    last_color: Color,
}

impl VisualizerObject {
    fn new(object_info: &ObjectPose, window: &mut Window) -> Self {
        let scene_node = attach_node_type(object_info.shape, window);
        let mut object = Self {
            node: scene_node,
            timeout: Duration::from_secs_f32(object_info.timeout),
            last_update: Instant::now(),
            current_shape: object_info.shape,
            last_pose: object_info.pose,
            last_rotation: object_info.rotation,
            last_color: object_info.color,
        };
        object.update_pose(object_info.pose);
        object.update_color(object_info.color);
        object
    }

    fn update(&mut self, update: &ObjectPose, window: &mut Window) {
        self.touch();
        self.update_pose(update.pose);
        self.update_rotation(update.rotation);
        self.update_color(update.color);
        self.timeout = Duration::from_secs_f32(update.timeout);
        self.update_shape(update.shape, window)
    }

    fn update_pose(&mut self, pose: (f32, f32, f32)) {
        self.last_pose = pose;
        if let Some(node) = &mut self.node {
            node.set_local_translation(na::Translation3::from(convert_coordinate_system(pose)));
        }
    }

    fn update_rotation(&mut self, rotation: (f32, f32, f32, f32)) {
        self.last_rotation = rotation;
        if let Some(node) = &mut self.node {
            node.set_local_rotation(convert_rotation_coordinate_system(rotation));
        }
    }

    fn update_color(&mut self, color: Color) {
        self.last_color = color;
        let color = color.to_rgb();
        if let Some(node) = &mut self.node {
            node.set_color(color.0, color.1, color.2);
        }
    }

    fn update_shape(&mut self, shape: Shape, window: &mut Window) {
        if self.current_shape != shape {
            self.current_shape = shape;
            if let Some(scene_node) = &mut self.node {
                scene_node.unlink()
            }
            self.node = attach_node_type(shape, window);
        }
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
        if let Some(scene_node) = &mut self.node {
            scene_node.unlink()
        }
    }
}

struct PointCloudContainer {
    point_cloud: PointCloud2,
    last_touched: Instant,
}

impl PointCloudContainer {
    fn new(point_cloud: PointCloud2) -> Self {
        Self {
            point_cloud,
            last_touched: Instant::now(),
        }
    }

    fn point_cloud(&self) -> &PointCloud2 {
        &self.point_cloud
    }

    fn is_timed_out(&self) -> bool {
        self.last_touched.elapsed() > Duration::from_secs_f32(self.point_cloud.timeout())
    }
}

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(long, default_value = "239.0.0.22:7072")]
    address: SocketAddrV4,
    #[clap(long, default_value = "239.0.0.22:7075")]
    point_cloud_address: SocketAddrV4,
    #[clap(long, default_value = "239.0.0.22:7076")]
    command_address: SocketAddrV4,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let pose_subscriber = PoseSubscriber::new(args.address).unwrap();
    let point_cloud_subscriber = PointCloudSubscriber::new(args.point_cloud_address).unwrap();
    let command_publisher = CommandPublisher::new(args.command_address).unwrap();
    let mut object_container = ObjectContainer::new();
    let mut window = Window::new("rustviz");

    window.set_background_color(0.1, 0.1, 0.1);
    window.set_light(Light::StickToCamera);
    add_ground_plane(&mut window);

    let mut camera = kiss3d::camera::ArcBall::new(
        na::Point3::new(1.0, 1.0, 1.0),
        na::Point3::new(0.0, 0.0, 0.0),
    );
    camera.set_dist_step(4.0);

    let mut last_projected_point: Option<na::Point3<f32>> = None;
    let mut last_button_down_pose: Option<na::Point3<f32>> = None;
    let mut command_id = 0_u32;
    let mut click_indicator = window.add_sphere(0.02);
    click_indicator.set_color(1.0, 1.0, 0.0);
    let mut release_indicator = window.add_sphere(0.01);
    release_indicator.set_color(0.0, 1.0, 1.0);

    let mut last_command = None;

    while !window.should_close() {
        // process window events
        for event in window.events().iter() {
            match event.value {
                WindowEvent::MouseButton(MouseButton::Button3, Action::Press, _modif) => {
                    if let Some(last_pose) = last_projected_point {
                        click_indicator.set_visible(true);
                        release_indicator.set_visible(false);
                        last_button_down_pose = Some(last_pose);
                        click_indicator.set_local_translation(last_pose.into());
                    }
                }
                WindowEvent::MouseButton(MouseButton::Button3, Action::Release, _modif) => {
                    if let Some(last_pose) = last_projected_point {
                        click_indicator.set_visible(true);
                        release_indicator.set_visible(true);
                        if let Some(last_button_down) = &last_button_down_pose {
                            let command = command_from_ground_plane_poses(
                                last_button_down,
                                &last_pose,
                                command_id,
                            );
                            last_command = Some(command);
                            command_id += 1;
                        }
                        last_button_down_pose = None;
                        release_indicator.set_local_translation(last_pose.into());
                    }
                }
                WindowEvent::CursorPos(x, y, _modif) => {
                    let mouse_position = na::Point2::new(x as f32, y as f32);
                    let window_size: na::Vector2<f32> = na::convert(window.size());
                    let (point, vector) = camera.unproject(&mouse_position, &window_size);
                    last_projected_point =
                        project_to_ground_plane_with_height(&point, &vector, 0.01);
                }
                _ => (),
            }
        }

        if let Some(command) = &last_command {
            if command_publisher.publish(command).is_err() {
                eprintln!("Failed to publish command");
            }
        }

        if let Some(last_button_down) = &last_button_down_pose {
            if let Some(last_project) = &last_projected_point {
                window.draw_line(
                    last_button_down,
                    last_project,
                    &na::Point3::new(1.0, 0.0, 1.0),
                )
            }
        }

        while let Ok(update) = pose_subscriber.next() {
            for object_update in update.updates() {
                object_container.update_object(object_update, &mut window);
            }
            for delete_id in update.deletions() {
                object_container.delete_object(delete_id);
            }
        }
        while let Ok(point_cloud_update) = point_cloud_subscriber.next() {
            object_container.update_point_clouds(point_cloud_update);
        }
        object_container.remove_timed_out();
        object_container.draw_lines(&mut window);
        object_container.draw_point_clouds(&mut window);
        window.draw_text(
            &object_container.display_message(),
            &na::Point2::new(1.0, 1.0),
            50.0,
            &kiss3d::text::Font::default(),
            &na::Point3::new(1.0, 1.0, 1.0),
        );
        window.render_with_camera(&mut camera);
    }
    window.close();
    Ok(())
}

fn project_to_ground_plane(
    position: &na::Point3<f32>,
    direction: &na::Vector3<f32>,
) -> Option<na::Point3<f32>> {
    let height = position.y;
    let angle = na::Rotation3::rotation_between(&na::Vector3::new(0., -1., 0.), direction);
    if let Some(angle) = angle {
        // could be map
        let angle = angle.angle();
        let vector_dist = height / angle.cos();
        Some(position + direction * vector_dist)
    } else {
        None
    }
}

fn project_to_ground_plane_with_height(
    position: &na::Point3<f32>,
    direction: &na::Vector3<f32>,
    height: f32,
) -> Option<na::Point3<f32>> {
    project_to_ground_plane(position, direction)
        .map(|point| na::Point3::new(point.x, point.y + height, point.z))
}

fn command_from_ground_plane_poses(
    origin: &na::Point3<f32>,
    target: &na::Point3<f32>,
    command_id: u32,
) -> Command {
    let origin_right_hand = inverse_convert_coordinate_system(&origin.coords);
    let target_right_hand = inverse_convert_coordinate_system(&target.coords);

    let transform_x = origin_right_hand.0 - target_right_hand.0;
    let transform_y = origin_right_hand.1 - target_right_hand.1;
    Command::new(
        command_id,
        (origin_right_hand.0, origin_right_hand.1),
        -transform_y.atan2(-transform_x),
        na::distance(origin, target),
    )
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

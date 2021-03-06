use clap::Clap;
use pose_publisher::{
    pose::{Color, Shape},
    PoseClientUpdate, PosePublisher, PosePublisherError,
};
use std::net::SocketAddrV4;

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(short, long, default_value = "239.0.0.22:7072")]
    address: SocketAddrV4,
}

fn main() -> Result<(), PosePublisherError> {
    let args = Args::parse();

    let pose_publisher = PosePublisher::new(args.address)?;
    for _ in 0..4 {
        for i in (0..=100).rev() {
            let i = i as f32;
            std::thread::sleep(std::time::Duration::from_secs_f32(0.02));
            let mut update = PoseClientUpdate::new();
            update
                .add("obj_a", (0., 0., 0.01 * i))
                .with_shape(Shape::Sphere(0.4));
            update
                .add("test line", (0., 0., 0.01 * i))
                .with_shape(Shape::Line((0., 0., 0.)));
            pose_publisher.publish(&update)?;
        }
        for i in 0..=100 {
            let i = i as f32;
            std::thread::sleep(std::time::Duration::from_secs_f32(0.02));
            let mut update = PoseClientUpdate::new();
            update
                .add("obj_a", (0., 0., 0.01 * i))
                .with_color(Color::Cyan)
                .with_shape(Shape::Cube(0.3, 0.01, 0.01));
            update
                .add("test line", (0., 0., 0.01 * i))
                .with_shape(Shape::Line((0., 0., 0.)))
                .with_color(Color::Magenta);
            pose_publisher.publish(&update)?;
        }
    }
    let mut update = PoseClientUpdate::new();
    update.delete("obj_a");
    pose_publisher.publish(&update)?;
    Ok(())
}

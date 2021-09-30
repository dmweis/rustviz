use clap::Clap;
use pose_publisher::{pose::Shape, PoseClientUpdate, PosePublisher, PosePublisherError};
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
    loop {
        std::thread::sleep(std::time::Duration::from_secs_f32(0.02));
        let mut update = PoseClientUpdate::new();
        update
            .add("rotated_object", (0., 0., 0.1))
            .with_shape(Shape::Cube(0.3, 0.01, 0.01))
            .with_rotation((0.9848, 0., 0.1736, 0.));
        pose_publisher.publish(update)?;
    }
}

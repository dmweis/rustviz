use clap::Clap;
use pose_publisher::{
    point_cloud::PointCloud2, pose::Color, PointCloudPublisher, PosePublisherError,
};
use std::net::SocketAddrV4;

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(short, long, default_value = "239.0.0.22:7075")]
    address: SocketAddrV4,
}

fn main() -> Result<(), PosePublisherError> {
    let args = Args::parse();

    let point_cloud_publisher = PointCloudPublisher::new(args.address)?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs_f32(0.2));
        let mut points = vec![];
        for i in 0..2000 {
            let i = i as f32 * 0.01;
            points.push((i.sin(), i.cos()));
        }

        let point_cloud = PointCloud2::from_points("example cloud", points)
            .with_color(Color::Cyan)
            .with_parent_frame_id("rotated_object");
        point_cloud_publisher.publish(point_cloud).unwrap();
    }
}

use clap::Clap;
use pose_publisher::{PosePublisherError, PoseSubscriber};
use std::net::SocketAddrV4;

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(short, long, default_value = "239.0.0.22:7072")]
    address: SocketAddrV4,
}

fn main() -> Result<(), PosePublisherError> {
    let args = Args::parse();

    let pose_subscriber = PoseSubscriber::new(args.address)?;
    loop {
        let new_poses = pose_subscriber.next()?;
        println!("New pose update {:?}", new_poses);
    }
}

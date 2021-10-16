pub mod commands;
mod multicast;
pub mod point_cloud;
pub mod pose;

use commands::Command;
use multicast::MulticastMessenger;
pub use point_cloud::PointCloud2;
pub use pose::{ObjectPose, PoseClientUpdate};
use std::{net::SocketAddrV4, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PosePublisherError {
    #[error("IP address needs to be in the multicast range")]
    AddressNotMulticast(SocketAddrV4),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
    #[error("Error converting data to string")]
    StringConversionError(#[from] std::str::Utf8Error),
    #[error("failed to parse json")]
    JsonParsingError,
}

type Result<T> = std::result::Result<T, PosePublisherError>;

#[derive(Clone)]
pub struct PosePublisher {
    messenger: Arc<MulticastMessenger>,
}

impl PosePublisher {
    pub fn new(multicast_address: SocketAddrV4) -> Result<Self> {
        let messenger = Arc::new(MulticastMessenger::new(multicast_address)?);
        Ok(Self { messenger })
    }

    pub fn publish(&self, update: &PoseClientUpdate) -> Result<()> {
        self.messenger.send(update)?;
        Ok(())
    }
}

pub struct PoseSubscriber {
    messenger: MulticastMessenger,
}

impl PoseSubscriber {
    pub fn new(multicast_address: SocketAddrV4) -> Result<Self> {
        let messenger = MulticastMessenger::new(multicast_address)?;
        Ok(Self { messenger })
    }

    pub fn next(&self) -> Result<PoseClientUpdate> {
        self.messenger.receive()
    }
}

pub struct PointCloudPublisher {
    messenger: MulticastMessenger,
}

impl PointCloudPublisher {
    pub fn new(multicast_address: SocketAddrV4) -> Result<Self> {
        let messenger = MulticastMessenger::new(multicast_address)?;
        Ok(Self { messenger })
    }

    pub fn publish(&self, point_cloud: &PointCloud2) -> Result<()> {
        self.messenger.send(point_cloud)?;
        Ok(())
    }
}

pub struct PointCloudSubscriber {
    messenger: MulticastMessenger,
}

impl PointCloudSubscriber {
    pub fn new(multicast_address: SocketAddrV4) -> Result<Self> {
        let messenger = MulticastMessenger::new(multicast_address)?;
        Ok(Self { messenger })
    }

    pub fn next(&self) -> Result<PointCloud2> {
        self.messenger.receive()
    }
}

pub struct CommandPublisher {
    messenger: MulticastMessenger,
}

impl CommandPublisher {
    pub fn new(multicast_address: SocketAddrV4) -> Result<Self> {
        let messenger = MulticastMessenger::new(multicast_address)?;
        Ok(Self { messenger })
    }

    pub fn publish(&self, command: &Command) -> Result<()> {
        self.messenger.send(command)?;
        Ok(())
    }
}

pub struct CommandSubscriber {
    messenger: MulticastMessenger,
}

impl CommandSubscriber {
    pub fn new(multicast_address: SocketAddrV4) -> Result<Self> {
        let messenger = MulticastMessenger::new(multicast_address)?;
        Ok(Self { messenger })
    }

    pub fn next(&self) -> Result<Command> {
        self.messenger.receive()
    }
}

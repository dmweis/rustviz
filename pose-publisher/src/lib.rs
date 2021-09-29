mod multicast;
pub mod pose;

use multicast::MulticastMessenger;
pub use pose::{ObjectPose, PoseClientUpdate};
use std::net::SocketAddrV4;
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

pub struct PosePublisher {
    messenger: MulticastMessenger,
}

impl PosePublisher {
    pub fn new(multicast_address: SocketAddrV4) -> Result<Self> {
        let messenger = MulticastMessenger::new(multicast_address)?;
        Ok(Self { messenger })
    }

    pub fn publish(&self, update: PoseClientUpdate) -> Result<()> {
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

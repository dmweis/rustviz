use crate::{PosePublisherError, Result};
use serde::{de::DeserializeOwned, Serialize};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddrV4;
use std::net::UdpSocket;
use std::str;

fn bind_multicast(addr: &SocketAddrV4, multi_addr: &SocketAddrV4) -> Result<UdpSocket> {
    // this code was inspired by https://github.com/henninglive/tokio-udp-multicast-chat
    if !multi_addr.ip().is_multicast() {
        return Err(PosePublisherError::AddressNotMulticast(*multi_addr));
    }
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&socket2::SockAddr::from(*addr))?;
    socket.set_multicast_loop_v4(true)?;
    socket.join_multicast_v4(multi_addr.ip(), addr.ip())?;
    Ok(socket.into())
}

const ALL_INTERFACES: [u8; 4] = [0, 0, 0, 0];

pub struct MulticastMessenger {
    socket: UdpSocket,
    multicast_address: SocketAddrV4,
}

impl MulticastMessenger {
    pub fn new(multicast_address: SocketAddrV4) -> Result<Self> {
        let addr = SocketAddrV4::new(ALL_INTERFACES.into(), multicast_address.port());
        let socket = bind_multicast(&addr, &multicast_address)?;
        socket.set_read_timeout(None)?;
        Ok(Self {
            socket,
            multicast_address,
        })
    }

    pub fn send<T: Serialize>(&self, message: &T) -> Result<()> {
        let payload = serde_json::to_string(message).unwrap();
        self.socket
            .send_to(payload.as_bytes(), self.multicast_address)?;
        Ok(())
    }

    pub fn receive<T: DeserializeOwned>(&self) -> Result<T> {
        let mut buf = [0; 65000];
        let len = self.socket.recv(&mut buf)?;
        let payload = str::from_utf8(&buf[..len])?;
        serde_json::from_str::<T>(payload).map_err(|_| PosePublisherError::JsonParsingError)
    }
}

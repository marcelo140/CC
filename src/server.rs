extern crate reverse_proxy;
extern crate bincode;

use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::Instant;
use self::reverse_proxy::packet::*;
use bincode::{serialize, Bounded};

pub struct Server {
    ip_addr: IpAddr,
    sequence: i32,
    responses : u32,
    last_sent : Instant,
    last_registration : Instant,
}

impl Server {
    pub fn new(ip: IpAddr) -> Server {
        Server {
            ip_addr: ip,
            sequence: 0,
            responses: 0,
            last_sent: Instant::now(),
            last_registration : Instant::now(),
        }
    }

    pub fn next_probe(&mut self) -> ProbeRequest {
        self.sequence += 1;
        self.last_sent = Instant::now();

        ProbeRequest::new(self.sequence - 1)
    }

    pub fn timeout(&self) -> bool {
        self.last_registration.elapsed().as_secs() > 5
    }

    pub fn send(&self, socket: &UdpSocket, message: Message) {
        let buffer = serialize(&message, Bounded(32)).unwrap();
        let addr = SocketAddr::new(self.ip_addr, 4000);

        let _ = socket.send_to(buffer.as_slice(), addr);
    }

    pub fn registrate(&mut self) {
        self.last_registration = Instant::now();
    }
}

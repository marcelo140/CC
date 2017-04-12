extern crate reverse_proxy;
extern crate bincode;

use bincode::{serialize, Bounded};
use self::reverse_proxy::packet::*;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::Instant;

pub enum ServerStatus {
    Green,
    Yellow,
    Red,
}

impl ServerStatus {
    fn deteriorate(self) -> ServerStatus {
        match self {
            ServerStatus::Green  => ServerStatus::Yellow,
            ServerStatus::Yellow => ServerStatus::Red,
            ServerStatus::Red    => ServerStatus::Red,
        }
    }
}

pub struct Server {
    ip_addr: IpAddr,
    sequence: u32,
    lost: u32,
    rtt: f64,
    load: f64,
    last_response: u32,
    last_sent : Instant,
    last_registration : Instant,
}

impl Server {
    pub fn new(ip: IpAddr) -> Server {
        Server {
            ip_addr: ip,
            sequence: 0,
            lost: 0,
            rtt: 0.0,
            load: 0.0,
            last_response: 0,
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

    pub fn get_rtt(&self) -> f64 {
        self.rtt
    }

    pub fn send(&self, socket: &UdpSocket, message: Message) {
        let buffer = serialize(&message, Bounded(64)).unwrap();
        let addr = SocketAddr::new(self.ip_addr, 4000);

        socket.send_to(buffer.as_slice(), addr).expect("Failed to end probe");
    }

    pub fn registrate(&mut self) {
        self.last_registration = Instant::now();
    }

    fn update_rtt(&self) -> f64 {
        let elapsed = self.last_sent.elapsed();
        let rtt_sample =
            (elapsed.as_secs() as f64) * 10f64.powi(9) + (elapsed.subsec_nanos() as f64);

        self.rtt*0.875 + rtt_sample*0.125
    }

    pub fn get_status(&self, avg_rtt: f64) -> ServerStatus {
        let mut status = ServerStatus::Green;

        if self.load > 0.7 {
            status = status.deteriorate();
        }

        if (self.lost as f64) / (self.sequence as f64) > 0.2 {
            status = status.deteriorate();
        }

        if self.rtt > avg_rtt {
            status = status.deteriorate();
        }

        return status;
    }

    pub fn response(&mut self, response: ProbeResponse) {
        if response.ack_number != self.sequence {
            return;
        }

        self.lost += (self.sequence - self.last_response) - 1;
        self.load = response.load;
        self.rtt = self.update_rtt();
        self.last_response = self.sequence;
    }
}

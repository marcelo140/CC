use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::Instant;
use packet::*;

pub struct Server {
    addr: SocketAddr,
    sequence: u32,
    lost: u32,
    rtt: f32,
    load: f32,
    connections: u32,
    last_response: u32,
    last_sent : Instant,
    last_registration : Instant,
}

impl Server {
    pub fn new(ip: IpAddr) -> Server {
        Server {
            addr: SocketAddr::new(ip, 5555),
            sequence: 0,
            lost: 0,
            rtt: 0.0,
            load: 0.0,
            connections: 0,
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
        self.last_registration.elapsed().as_secs() > 6
    }

    pub fn get_conn(&self) -> u32 {
        self.connections
    }

    pub fn inc_connections(&mut self) {
        self.connections += 1;
    }

    pub fn dec_connections(&mut self) {
        self.connections -= 1;
    }

    pub fn get_load(&self) -> f32 {
        self.load
    }

    pub fn get_rtt(&self) -> f32 {
        self.rtt
    }

    pub fn get_lost(&self) -> f32 {
        (self.lost as f32) / (self.sequence as f32)
    }

    pub fn send(&self, socket: &UdpSocket, message: Message) {
        let buffer = message.serialize().unwrap();
        socket.send_to(buffer.as_slice(), self.addr).expect("Failed to send probe");
    }

    pub fn registrate(&mut self) {
        self.last_registration = Instant::now();
    }

    fn update_rtt(&self) -> f32 {
        let elapsed = self.last_sent.elapsed();
        let rtt_sample =
            (elapsed.as_secs() as f32) * 10f32.powi(9) + (elapsed.subsec_nanos() as f32);

        self.rtt*0.875 + rtt_sample*0.125
    }

    pub fn get_status(&self, maximuns: (f32, f32, f32, u32)) -> f32 {
        let mut status = 0f32;
        let (load, lost, rtt, connections) = maximuns;

        if load != 0.0 {
            status += self.load / load;
        }

        if lost != 0.0 {
            status += self.get_lost() / lost;
        }

        if rtt != 0.0 {
            status += self.rtt / rtt;
        }

        if connections != 0 {
            status += (self.connections as f32) / (connections as f32);
        }

        return status;
    }

    pub fn handle_response(&mut self, response: ProbeResponse) {
        if response.ack_number != self.sequence {
            return;
        }

        self.lost += (self.sequence - self.last_response) - 1;
        self.load = response.load;
        self.rtt = self.update_rtt();
        self.last_response = self.sequence;
    }
}

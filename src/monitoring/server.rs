use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::Instant;
use packet::*;
use monitoring::ServerStatus;

pub struct Server {
    addr: SocketAddr,
    sequence: u32,
    lost: u32,
    rtt: f64,
    load: f64,
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
        self.last_registration.elapsed().as_secs() > 5
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

    pub fn get_rtt(&self) -> f64 {
        self.rtt
    }

    pub fn get_lost(&self) -> f64 {
        (self.lost as f64) / (self.sequence as f64)
    }

    pub fn send(&self, socket: &UdpSocket, message: Message) {
        let buffer = message.serialize().unwrap();
        socket.send_to(buffer.as_slice(), self.addr).expect("Failed to send probe");
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

    pub fn get_status(&self, avg_rtt: f64, avg_lost: f64, avg_conn: u32) -> ServerStatus {
        let mut status = ServerStatus::Green;

        if self.load > 0.7 {
            status = status.deteriorate();
        }

        if self.get_lost() > avg_lost {
            status = status.deteriorate();
        }

        if self.rtt > avg_rtt {
            status = status.deteriorate();
        }

        if self.connections > avg_conn {
            status = status.deteriorate();
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

extern crate reverse_proxy;

use reverse_proxy::packet::*;
use std::net::{UdpSocket, IpAddr};
use std::collections::BTreeMap;
use std::sync::Mutex;
use server::{ServerStatus, Server};

pub struct Monitor {
    servers: Mutex<BTreeMap<IpAddr, Server>>
}

impl Monitor {
    pub fn new() -> Monitor {
        Monitor {
            servers: Mutex::new(BTreeMap::<IpAddr, Server>::new()),
        }
    }

    pub fn registrate(&self, addr: IpAddr) {
        let mut servers = self.servers.lock().unwrap();
        let mut server = servers.entry(addr).or_insert(Server::new(addr));

        server.registrate();
    }

    pub fn send_probes(&self, socket: &UdpSocket) {
        let mut servers = self.servers.lock().unwrap();
        let mut removable = Vec::new();

        for (ip_addr, server) in servers.iter_mut() {
            if server.timeout() {
                removable.push(*ip_addr);
                continue;
            }

            let message = server.next_probe().into_message().unwrap();
            server.send(socket, message);
        }

        for server in removable {
            servers.remove(&server);
        }
    }

    pub fn receive(&self, addr: IpAddr, response: ProbeResponse) {
        let mut servers = self.servers.lock().unwrap();

        if let Some(record) = servers.get_mut(&addr) {
            record.response(response);
        }
    }

    pub fn pick_server(&self) -> Option<IpAddr> {
        let (mut red, mut yellow) = (None, None);
        let avg = self.avg_rtt();
        let servers = self.servers.lock().unwrap();

        for (ip, s) in &(*servers) {
            match s.get_status(avg) {
                ServerStatus::Green => return Some(*ip),
                ServerStatus::Yellow => yellow = Some(*ip),
                ServerStatus::Red => red = Some(*ip),
            }
        }

        match yellow {
            Some(_) => return yellow,
            None => return red,
        }
    }

    fn avg_rtt(&self) -> f64 {
        let servers = self.servers.lock().unwrap();

        let sum = servers.values().fold(0f64, |acc, ref s| acc + s.get_rtt());
        let avg = sum / (servers.len() as f64);

        return avg;
    }
}

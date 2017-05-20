use std::net::{UdpSocket, IpAddr};
use std::collections::BTreeMap;
use std::sync::Mutex;
use packet::*;
use monitoring::Server;
use monitoring::ServerStatus;

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

    pub fn inc_connections(&self, addr: IpAddr) {
        let mut servers = self.servers.lock().unwrap();
        match servers.get_mut(&addr) {
            Some(s) => s.inc_connections(),
            _ => println!("Server is no longer available"),
        }
    }

    pub fn dec_connections(&self, addr: IpAddr) {
        let mut servers = self.servers.lock().unwrap();
        match servers.get_mut(&addr) {
            Some(s) => s.dec_connections(),
            _ => println!("Server is no longer available"),
        }
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

    pub fn handle_response(&self, addr: IpAddr, response: ProbeResponse) {
        let mut servers = self.servers.lock().unwrap();

        if let Some(record) = servers.get_mut(&addr) {
            record.handle_response(response);
        }
    }

    pub fn pick_server(&self) -> Option<IpAddr> {
        let (mut red, mut yellow) = (None, None);
        let (rtt, lost, conn) = self.calculate_averages();
        let servers = self.servers.lock().unwrap();

        for (ip, s) in &(*servers) {
            match s.get_status(rtt, lost, conn) {
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

    fn calculate_averages(&self) -> (f64, f64, u32) {
        let servers = self.servers.lock().unwrap();

        let rtt = servers.values().fold(0f64, |acc, ref s| acc + s.get_rtt());
        let lost = servers.values().fold(0f64, |acc, ref s| acc + s.get_lost());
        let conn = servers.values().fold(0u32, |acc, ref s| acc + s.get_conn());

        let avg_rtt = rtt / (servers.len() as f64);
        let avg_lost = lost / (servers.len() as f64);
        let avg_conn = conn / (servers.len() as u32);

        return (avg_rtt, avg_lost, avg_conn);
    }
}

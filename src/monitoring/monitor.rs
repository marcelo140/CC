use std::cmp::Ordering::Equal;
use std::net::{UdpSocket, IpAddr};
use std::collections::BTreeMap;
use std::sync::Mutex;
use packet::*;
use monitoring::Server;

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

    pub fn clean_unregisted(&self) {
        let mut servers = self.servers.lock().unwrap();
        let mut removable = Vec::new();

        for (ip_addr, server) in servers.iter() {
            if server.timeout() {
                removable.push(*ip_addr);
            }
        }

        for server in removable {
            servers.remove(&server);
        }
    }

    pub fn send_probes(&self, socket: &UdpSocket) {
        let mut servers = self.servers.lock().unwrap();

        for (ip_addr, server) in servers.iter_mut() {
            let message = server.next_probe().into_message().unwrap();
            server.send(socket, message);
        }
    }

    pub fn handle_response(&self, addr: IpAddr, response: ProbeResponse) {
        let mut servers = self.servers.lock().unwrap();

        if let Some(record) = servers.get_mut(&addr) {
            record.handle_response(response);
        }
    }

    pub fn pick_server(&self) -> Option<IpAddr> {
        let maximums = self.get_maximums();
        let servers = self.servers.lock().unwrap();
        let mut aux = Vec::new();

        for (ip, s) in &(*servers) {
            aux.push((ip, s.get_status(maximums)));
        }

        aux.sort_by(|x,y| (x.1).partial_cmp(&y.1).unwrap_or(Equal));
        match aux.first() {
            Some(v) => Some(*v.0),
            None => None,
        }
    }

    fn get_maximums(&self) -> (f32, f32, f32, u32) {
        let servers = self.servers.lock().unwrap();

        let max_load = servers.values().map(|s| s.get_load())
            .max_by(|x,y| x.partial_cmp(y).unwrap_or(Equal)).unwrap_or(0.0);
        let max_rtt = servers.values().map(|s| s.get_rtt())
            .max_by(|x,y| x.partial_cmp(y).unwrap_or(Equal)).unwrap_or(0.0);
        let max_lost = servers.values().map(|s| s.get_lost())
            .max_by(|x,y| x.partial_cmp(y).unwrap_or(Equal)).unwrap_or(0.0);
        let max_conn = servers.values().map(|s| s.get_conn())
            .max_by(|x,y| x.partial_cmp(y).unwrap_or(Equal)).unwrap_or(0);

        return (max_load, max_lost, max_rtt, max_conn);
    }
}

extern crate reverse_proxy;

use reverse_proxy::packet::*;
use std::net::{UdpSocket, IpAddr};
use std::collections::BTreeMap;
use std::sync::Mutex;
use server::Server;

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

            let message = server.next_probe().into_message();
            server.send(socket, message);
        }

        for server in removable {
            servers.remove(&server);
        }
    }
}
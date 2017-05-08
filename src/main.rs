extern crate bincode;
extern crate reverse_proxy;

mod server;
mod monitor;

use std::env;
use std::thread;
use std::sync::Arc;
use monitor::Monitor;
use std::time::Duration;
use std::net::UdpSocket;
use bincode::deserialize;
use reverse_proxy::packet::*;

fn start_receiver(socket: UdpSocket, servers: &mut Arc<Monitor>) {
    let mut buffer = [0; 64];

    loop {
        let (_, tx) = socket.recv_from(&mut buffer).unwrap();
        let data: Message = deserialize(&buffer).unwrap();

        match data.flag {
            MessageType::ProbeResponse => {
                let response = ProbeResponse::from_message(data);
                servers.receive(tx.ip(), response);
            },
            MessageType::Registration  => {
                servers.registrate(tx.ip());
            },
            MessageType::ProbeRequest  => unreachable!(),
        }
    }
}

fn start_sender(socket: UdpSocket, monitor: &mut Arc<Monitor>) {
    loop {
        monitor.send_probes(&socket);
        thread::sleep(Duration::from_secs(3));

        match monitor.pick_server() {
            Some(ip) => println!("Selected server {}!", ip),
            None => println!("No servers available!"),
        }
    }
}

fn main() {
    let mut servers = Arc::new(Monitor::new());

    let socket = match env::args().nth(1) {
        None => panic!("No IP address was provided for listening"),
        Some(ip) => UdpSocket::bind((ip.as_str(), 5555))
                              .expect("Failed while binding to the specified address"),
    };

    {
        let socket = socket.try_clone().expect("Failed while starting sender thread");
        let mut servers = servers.clone();

        thread::spawn(move || {
            start_sender(socket, &mut servers);
        });
    }

    start_receiver(socket, &mut servers);
}

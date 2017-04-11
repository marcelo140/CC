extern crate bincode;
extern crate reverse_proxy;

mod server;
mod monitor;

use std::thread;
use std::sync::Arc;
use std::time::Duration;
use std::net::UdpSocket;
use monitor::Monitor;
use bincode::deserialize;
use reverse_proxy::packet::*;

fn start_receiver(socket: UdpSocket, servers: &mut Arc<Monitor>) {
    let mut buffer = [0; 30];

    loop {
        let (_, tx) = socket.recv_from(&mut buffer).expect("Didn't receive data");
        let data: Message = deserialize(&buffer).unwrap();

        match data.flag {
            MessageType::ProbeResponse => continue,
            MessageType::Registration  => servers.registrate(tx.ip()),
            MessageType::ProbeRequest  => unreachable!(),
        }
    }
}

fn start_sender(socket: UdpSocket, servers: &mut Arc<Monitor>) {
    loop {
        servers.send_probes(&socket);
        thread::sleep(Duration::from_secs(3));
    }
}

fn main() {
    let socket = UdpSocket::bind("localhost:5555").expect("Couldn't bind to address");
    let mut servers = Arc::new(Monitor::new());

    {
        let socket = socket.try_clone().expect("Couldn't obtain socket's copy");
        let mut servers = servers.clone();

        thread::spawn(move || {
            start_sender(socket, &mut servers);
        });
    }

    start_receiver(socket, &mut servers);
}

extern crate bincode;
extern crate reverse_proxy;

mod server;
mod monitor;

use std::env;
use std::thread;
use std::sync::Arc;
use monitor::Monitor;
use std::time::Duration;
use std::io::{Read, Write};
use reverse_proxy::packet::*;
use std::net::{TcpListener, TcpStream, UdpSocket};

macro_rules! exit {
    ($msg:expr) => {{ println!($msg); return }};
}

fn start_receiver(socket: UdpSocket, servers: &mut Arc<Monitor>) {
    let mut buffer = [0; 64];

    loop {
        let (_, tx) = socket.recv_from(&mut buffer).unwrap();
        let data: Message = Message::deserialize(&buffer).unwrap();

        match data.flag {
            MessageType::ProbeResponse => {
                let res = ProbeResponse::from_message(data).unwrap();
                servers.receive(tx.ip(), res);
            },
            MessageType::Registration => servers.registrate(tx.ip()),
            MessageType::ProbeRequest  => unreachable!(),
        }
    }
}

fn start_sender(socket: UdpSocket, monitor: &mut Arc<Monitor>) {
    loop {
        monitor.send_probes(&socket);
        thread::sleep(Duration::from_secs(3));
    }
}

fn start_listener(monitor: &mut Arc<Monitor>) {
    let listener = match env::args().nth(1) {
        None => exit!("No IP address provided for listening"),
        Some(ip) => TcpListener::bind((ip.as_str(), 80)).unwrap(),
    };

    loop {
        let mut client = match listener.accept() {
            Ok((stream, _)) => stream,
            _ => continue,
        };

        let mut server = match monitor.pick_server() {
            Some(ip) => TcpStream::connect((ip, 80)).unwrap(),
            None => {
                println!("No available server!");
                continue;
            },
        };

        handle_connection(&mut client, &mut server);
    }
}

fn forward(from: &mut TcpStream, to: &mut TcpStream) {
    let mut buffer = [0; 4096];

    loop {
        let _ = match from.read(&mut buffer) {
            Ok(s) => s,
            Err(_) => break,
        };

        let _ = match to.write(&buffer) {
            Ok(s) => s,
            Err(_) => break,
        };
    }
}

fn handle_connection(client: &mut TcpStream, server: &mut TcpStream) {
    {
        let mut client = client.try_clone().unwrap();
        let mut server = server.try_clone().unwrap();

        thread::spawn(move || {
            forward(&mut client, &mut server);
        });
    }

    {
        let mut client = client.try_clone().unwrap();
        let mut server = server.try_clone().unwrap();

        thread::spawn(move || {
            forward(&mut server, &mut client);
        });
    }
}

fn main() {
    let mut servers = Arc::new(Monitor::new());

    let socket = match env::args().nth(1) {
        None => exit!("No IP address provided for listening"),
        Some(ip) => UdpSocket::bind((ip.as_str(), 5555)).unwrap(),
    };

    {
        let socket = socket.try_clone().expect("Failed while starting sender thread");
        let mut servers = servers.clone();

        thread::spawn(move || {
            start_sender(socket, &mut servers);
        });
    }

    {
        let mut servers = servers.clone();

        thread::spawn(move || {
            start_listener(&mut servers);
        });
    }

    start_receiver(socket, &mut servers);
}

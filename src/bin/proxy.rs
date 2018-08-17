extern crate reverse_proxy;

use reverse_proxy::monitoring::*;
use reverse_proxy::packet::*;
use std::env;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

macro_rules! exit {
    ($msg:expr) => {{
        println!($msg);
        return;
    }};
}

fn start_receiver(socket: &UdpSocket, servers: &mut Arc<Monitor>) {
    let mut buffer = [0; 64];

    loop {
        let (_, tx) = socket.recv_from(&mut buffer).unwrap();
        let data: Message = Message::deserialize(&buffer).unwrap();

        match data.flag {
            MessageType::ProbeResponse => {
                let res = ProbeResponse::from_message(data).unwrap();
                servers.handle_response(tx.ip(), res);
            }
            MessageType::Registration => servers.registrate(tx.ip()),
            MessageType::ProbeRequest => unreachable!(),
        }
    }
}

fn start_sender(socket: &UdpSocket, monitor: &mut Arc<Monitor>) {
    loop {
        monitor.clean_unregisted();
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

        monitor.clean_unregisted();
        let mut server = match monitor.pick_server() {
            Some(ip) => {
                println!("Selected ip: {}", ip);
                monitor.inc_connections(ip);
                TcpStream::connect((ip, 80)).unwrap()
            }
            None => {
                println!("No available server!");
                continue;
            }
        };

        let mut _monitor = monitor.clone();
        handle_connection(&mut _monitor, &mut client, &mut server);
    }
}

fn handle_connection(monitor: &mut Arc<Monitor>, client: &mut TcpStream, server: &mut TcpStream) {
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
        let monitor = monitor.clone();

        thread::spawn(move || {
            forward(&mut server, &mut client);
            monitor.dec_connections(server.peer_addr().unwrap().ip())
        });
    }
}

fn forward(from: &mut TcpStream, to: &mut TcpStream) {
    let mut buffer = [0; 4096];

    loop {
        let size = match from.read(&mut buffer) {
            Ok(0) => break,
            Ok(s) => s,
            Err(e) => {
                println!("Error while reading from TCP socket: {}", e.description());
                break;
            }
        };

        let (content, _) = buffer.split_at(size);

        let _ = match to.write(&content) {
            Ok(s) => s,
            Err(e) => {
                println!("Error while writing to TCP socket: {}", e.description());
                break;
            }
        };
    }
}

fn main() {
    let mut servers = Arc::new(Monitor::new());

    let socket = match env::args().nth(1) {
        None => exit!("No IP address provided for listening"),
        Some(ip) => UdpSocket::bind((ip.as_str(), 5555)).unwrap(),
    };

    {
        let socket = socket
            .try_clone()
            .expect("Failed while starting sender thread");
        let mut servers = servers.clone();

        thread::spawn(move || {
            start_sender(&socket, &mut servers);
        });
    }

    {
        let mut servers = servers.clone();

        thread::spawn(move || {
            start_listener(&mut servers);
        });
    }

    start_receiver(&socket, &mut servers);
}

extern crate reverse_proxy;

use std::env;
use std::thread;
use std::error::Error;
use std::net::UdpSocket;
use std::time::Duration;
use reverse_proxy::packet::*;

macro_rules! exit {
    ($msg:expr) => {{ println!($msg); return }};
}

fn process_request(buffer: &[u8]) -> Result<ProbeResponse, MsgErr> {
    ProbeRequest::from_bytes(buffer).map(ProbeResponse::from_request)
}

fn start_receiver(socket : UdpSocket) {
    let mut buffer = [0; 64];

    loop {
        socket.recv_from(&mut buffer).unwrap();
        let res = process_request(&buffer).and_then(|r| r.into_bytes());

        let _ = match res {
            Ok(res) => socket.send(res.as_slice()),
            Err(e) => {
                println!("Could not handle request: {}", e.description());
                continue;
            }
        };
    }
}

fn send_registration(socket: &UdpSocket) {
    let registration = Message::new_registration().unwrap();
    let _ = socket.send(registration.as_slice());
}

fn start_registrator(socket: UdpSocket) {
    loop {
        send_registration(&socket);
        thread::sleep(Duration::from_secs(3));
    }
}

fn main() {
    let socket = match env::args().nth(1) {
        None => exit!("No IP address provided for listening"),
        Some(ip) => UdpSocket::bind((ip.as_str(), 5555)).unwrap(),
    };

    match env::args().nth(2) {
        None => exit!("No proxy IP address provided"),
        Some(ip) => socket.connect((ip.as_str(), 5555)).unwrap(),
    };

    {
        let socket = socket.try_clone().expect("Couldn't get a socket copy");

        thread::spawn(move || {
            start_receiver(socket);
        });
    }

    start_registrator(socket);
}

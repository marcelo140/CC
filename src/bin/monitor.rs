extern crate bincode;
extern crate reverse_proxy;

use std::thread;
use std::net::UdpSocket;
use std::time::Duration;
use reverse_proxy::packet::*;
use bincode::{serialize, deserialize, Bounded};

fn process_request(buffer: &[u8]) -> Message {
    let message : Message = deserialize(&buffer).unwrap();
    let request  = ProbeRequest::from_message(message);

    ProbeResponse::from_request(request).into_message()
}

fn start_receiver(socket : UdpSocket) {
    let mut buffer = [0; 255];

    loop {
        let _ = socket.recv(&mut buffer);
        let message = process_request(&buffer);

        let encoded = serialize(&message, Bounded(64)).unwrap();
        let _ = socket.send_to(encoded.as_slice(), "localhost:5555");
    }
}

fn send_registration(socket: &UdpSocket) {
    let message = Message::new(MessageType::Registration, vec![]);
    let encoded = serialize(&message, Bounded(64)).unwrap();

    let _ = socket.send_to(encoded.as_slice(), "localhost:5555");
}

fn start_registrator(socket: UdpSocket) {
    loop {
        send_registration(&socket);
        thread::sleep(Duration::from_secs(3));
    }
}

fn main() {
    let socket = UdpSocket::bind("localhost:4000").expect("Couldn't bind to adress");

    {
        let socket = socket.try_clone().expect("Couldn't get a socket copy");

        thread::spawn(move || {
            start_receiver(socket);
        });
    }

    start_registrator(socket);
}

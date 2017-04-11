use packet::message::*;
use bincode::{serialize, deserialize, Bounded};

#[derive(Serialize, Deserialize)]
pub struct Registration {
    sequence_number : i32,
}

impl Registration {
    pub fn new(seq: i32) -> Registration {
        Registration {
            sequence_number: seq,
        }
    }
}

impl IntoMessage for Registration {
    fn into_message(self) -> Message {
        let mut content = Vec::new();
        content.extend_from_slice(&serialize(&self, Bounded(64)).unwrap());

        Message::new(MessageType::Registration, content)
    }
}

impl FromMessage for Registration {
    fn from_message(m: Message) -> Registration {
        deserialize(&m.content).unwrap()
    }
}

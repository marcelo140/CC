use packet::message::*;
use bincode::{serialize, deserialize, Bounded};

#[derive(Serialize, Deserialize)]
pub struct ProbeRequest {
    sequence_number : i32,
}

impl ProbeRequest {
    pub fn new(seq: i32) -> ProbeRequest {
        ProbeRequest {
            sequence_number: seq,
        }
    }

    pub fn sequence(&self) -> i32 {
        self.sequence_number
    }
}

impl IntoMessage for ProbeRequest {
    fn into_message(self) -> Message {
        let mut content = Vec::new();
        content.extend_from_slice(&serialize(&self, Bounded(64)).unwrap());

        Message::new(MessageType::ProbeRequest, content)
    }
}

impl FromMessage for ProbeRequest {
    fn from_message(m: Message) -> ProbeRequest {
        deserialize(&m.content).unwrap()
    }
}

use packet::message::*;
use packet::probe_request::ProbeRequest;
use bincode::{serialize, deserialize, Bounded};

#[derive(Serialize, Deserialize)]
pub struct ProbeResponse {
    ack_number: i32,
    load_average: f32,
}

impl ProbeResponse {
    pub fn new(ack : i32, load: f32) -> ProbeResponse {
        ProbeResponse {
            ack_number: ack,
            load_average: load,
        }
    }

    pub fn from_request(request: ProbeRequest) -> ProbeResponse {
        ProbeResponse {
            ack_number: request.sequence() + 1,
            load_average: 0.0,
        }
    }

    pub fn ack(&self) -> i32 {
        self.ack_number
    }
}

impl IntoMessage for ProbeResponse {
    fn into_message(self) -> Message {
        let mut content = Vec::new();
        content.extend_from_slice(&serialize(&self, Bounded(64)).unwrap());

        Message::new(MessageType::ProbeResponse, content)
    }
}

impl FromMessage for ProbeResponse {
    fn from_message(m: Message) -> ProbeResponse {
        deserialize(&m.content).unwrap()
    }
}

extern crate sys_info;

use packet::message::*;
use packet::probe_request::ProbeRequest;
use bincode::{serialize, deserialize, Bounded};

#[derive(Serialize, Deserialize)]
pub struct ProbeResponse {
    pub ack_number: u32,
    pub load: f64,
}

impl ProbeResponse {
    pub fn from_request(request: ProbeRequest) -> ProbeResponse {
        let load = sys_info::loadavg().unwrap();
        let cpu_num = sys_info::cpu_num().unwrap();

        ProbeResponse {
            ack_number: request.sequence + 1,
            load: load.one / (cpu_num as f64),
        }
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

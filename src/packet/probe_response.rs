extern crate sys_info;

use packet::message::*;
use packet::probe_request::ProbeRequest;
use bincode::{serialize, deserialize, Bounded};

#[derive(Serialize, Deserialize)]
pub struct ProbeResponse {
    pub ack_number: u32,
    pub load: f32,
}


impl ProbeResponse {
    pub fn from_request(request: ProbeRequest) -> ProbeResponse {
        let load = sys_info::loadavg().unwrap();
        let cpu_num = sys_info::cpu_num().unwrap();

        ProbeResponse {
            ack_number: request.sequence + 1,
            load: (load.one as f32) / (cpu_num as f32),
        }
    }
}

impl SerializablePacket<ProbeResponse> for ProbeResponse {
    fn into_message(self) -> Result<Message, MsgErr> {
        serialize(&self, Bounded(64))
            .map(|content| Message::new(MessageType::ProbeResponse, content))
    }

    fn from_message(message: Message) -> Result<ProbeResponse, MsgErr> {
        deserialize(&message.content)
    }

    fn into_bytes(self) -> Result<Vec<u8>, MsgErr> {
        serialize(&self, Bounded(64))
            .map(|content| Message::new(MessageType::ProbeResponse, content))
            .and_then(|message| serialize(&message, Bounded(64)))
    }

    fn from_bytes(buffer: &[u8]) -> Result<ProbeResponse, MsgErr> {
        deserialize(&buffer).and_then(|msg: Message|
            deserialize(&msg.content)
        )
    }
}

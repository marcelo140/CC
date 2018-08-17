use bincode::{deserialize, serialize, Bounded};
use packet::message::*;

#[derive(Serialize, Deserialize)]
pub struct ProbeRequest {
    pub sequence: u32,
}

impl ProbeRequest {
    pub fn new(seq: u32) -> ProbeRequest {
        ProbeRequest { sequence: seq }
    }
}

impl SerializablePacket<ProbeRequest> for ProbeRequest {
    fn into_message(self) -> Result<Message, MsgErr> {
        serialize(&self, Bounded(64))
            .map(|content| Message::new(MessageType::ProbeRequest, content))
    }

    fn from_message(message: Message) -> Result<ProbeRequest, MsgErr> {
        deserialize(&message.content)
    }

    fn into_bytes(self) -> Result<Vec<u8>, MsgErr> {
        serialize(&self, Bounded(64))
            .map(|content| Message::new(MessageType::ProbeRequest, content))
            .and_then(|message| serialize(&message, Bounded(64)))
    }

    fn from_bytes(buffer: &[u8]) -> Result<ProbeRequest, MsgErr> {
        deserialize(&buffer).and_then(|msg: Message| deserialize(&msg.content))
    }
}

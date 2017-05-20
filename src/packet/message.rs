use bincode::{deserialize, serialize, Bounded, Error};

pub type MsgErr = Error;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Registration,
    ProbeRequest,
    ProbeResponse,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub flag : MessageType,
    pub content : Vec<u8>,
}

impl Message {
    pub fn new(flag: MessageType, content: Vec<u8>) -> Message {
        Message {
            flag: flag,
            content: content,
        }
    }

    pub fn new_registration() -> Result<Vec<u8>, MsgErr> {
        let message = Message {
            flag: MessageType::Registration,
            content: vec![],
        };

        serialize(&message, Bounded(64))
    }

    pub fn serialize(self) -> Result<Vec<u8>, MsgErr> {
        serialize(&self, Bounded(64))
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Message, MsgErr> {
        deserialize(&buffer)
    }
}

pub trait SerializablePacket<T> {
    fn into_message(self) -> Result<Message, MsgErr>;
    fn from_message(buffer: Message) -> Result<T, MsgErr>;

    fn into_bytes(self) -> Result<Vec<u8>, MsgErr>;
    fn from_bytes(buffer: &[u8]) -> Result<T, MsgErr>;
}

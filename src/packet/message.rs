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
}

pub trait IntoMessage {
    fn into_message(self) -> Message;
}

pub trait FromMessage {
    fn from_message(m: Message) -> Self;
}

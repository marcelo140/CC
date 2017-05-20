pub enum ServerStatus {
    Green,
    Yellow,
    Red,
}

impl ServerStatus {
    pub fn deteriorate(self) -> ServerStatus {
        match self {
            ServerStatus::Green  => ServerStatus::Yellow,
            ServerStatus::Yellow => ServerStatus::Red,
            ServerStatus::Red    => ServerStatus::Red,
        }
    }
}


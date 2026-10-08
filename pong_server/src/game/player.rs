use crate::net::socket::BerkeleyStream;

#[allow(dead_code)]
pub struct Player {
    pub id: u8,
    pub nickname: String,
    pub email: String,
    pub stream: BerkeleyStream,
}

impl Player {
    pub fn new(id: u8, nickname: String, email: String, stream: BerkeleyStream) -> Self {
        Self {
            id,
            nickname,
            email,
            stream,
        }
    }
}

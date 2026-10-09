#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    RegisterReq = 0x01,
    RegisterResp = 0x02,
    WaitMatch = 0x03,
    GameStart = 0x04,
    MoveInput = 0x05,
    GameState = 0x06,
    GameOver = 0x07,
    Unknown = 0xFF,
}

impl From<u8> for OpCode {
    fn from(val: u8) -> Self {
        match val {
            0x01 => OpCode::RegisterReq,
            0x02 => OpCode::RegisterResp,
            0x03 => OpCode::WaitMatch,
            0x04 => OpCode::GameStart,
            0x05 => OpCode::MoveInput,
            0x06 => OpCode::GameState,
            0x07 => OpCode::GameOver,
            _ => OpCode::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Still = 0,
    Up = 1,
    Down = 2,
}

/// Estado empaquetado del juego a retransmitir (10 bytes de payload)
#[derive(Debug, Clone, Copy)]
pub struct GameStatePayload {
    pub paddle1_y: u16,
    pub paddle2_y: u16,
    pub ball_x: u16,
    pub ball_y: u16,
    pub score_p1: u8,
    pub score_p2: u8,
}

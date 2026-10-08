use super::packet::{Direction, GameStatePayload, OpCode};

pub struct Codec;

impl Codec {
    /// Decodifica la cabecera fija de 3 bytes
    pub fn decode_header(buf: &[u8; 3]) -> (OpCode, u16) {
        let opcode = OpCode::from(buf[0]);
        let len = u16::from_be_bytes([buf[1], buf[2]]);
        (opcode, len)
    }

    /// Codifica la cabecera fija de 3 bytes
    pub fn encode_header(opcode: OpCode, payload_len: u16) -> [u8; 3] {
        let len_bytes = payload_len.to_be_bytes();
        [opcode as u8, len_bytes[0], len_bytes[1]]
    }

    /// Decodifica petición de registro: Nick_Len(1B) + Nick + Email_Len(1B) + Email
    pub fn decode_register_req(payload: &[u8]) -> Result<(String, String), String> {
        if payload.len() < 2 {
            return Err("Payload de registro demasiado corto".to_string());
        }

        let nick_len = payload[0] as usize;
        if payload.len() < 1 + nick_len + 1 {
            return Err("Longitud de nick inválida".to_string());
        }

        let nick = String::from_utf8_lossy(&payload[1..1 + nick_len]).to_string();
        let email_len_idx = 1 + nick_len;
        let email_len = payload[email_len_idx] as usize;

        if payload.len() < email_len_idx + 1 + email_len {
            return Err("Longitud de email inválida".to_string());
        }

        let email = String::from_utf8_lossy(&payload[email_len_idx + 1..email_len_idx + 1 + email_len]).to_string();
        Ok((nick, email))
    }

    /// Codifica respuesta de registro: [0x02] [0x00, 0x02] [Status, PlayerID]
    pub fn encode_register_resp(status: u8, player_id: u8) -> Vec<u8> {
        let mut msg = Vec::with_capacity(5);
        msg.extend_from_slice(&Self::encode_header(OpCode::RegisterResp, 2));
        msg.push(status);
        msg.push(player_id);
        msg
    }

    /// Codifica espera de rival: [0x03] [0x00, 0x00]
    pub fn encode_wait_match() -> Vec<u8> {
        Self::encode_header(OpCode::WaitMatch, 0).to_vec()
    }

    /// Codifica inicio de partida: [0x04] [0x00, 0x01] [Role: 1 o 2]
    pub fn encode_game_start(role: u8) -> Vec<u8> {
        let mut msg = Vec::with_capacity(4);
        msg.extend_from_slice(&Self::encode_header(OpCode::GameStart, 1));
        msg.push(role);
        msg
    }

    /// Decodifica comando de movimiento: [Direction: 1B]
    pub fn decode_move_input(payload: &[u8]) -> Direction {
        if payload.is_empty() {
            Direction::Still
        } else {
            Direction::from(payload[0])
        }
    }

    /// Codifica estado del juego (10 bytes de payload):
    /// Paddle1_Y(2B), Paddle2_Y(2B), Ball_X(2B), Ball_Y(2B), Score1(1B), Score2(1B)
    pub fn encode_game_state(state: &GameStatePayload) -> Vec<u8> {
        let mut msg = Vec::with_capacity(13);
        msg.extend_from_slice(&Self::encode_header(OpCode::GameState, 10));
        msg.extend_from_slice(&state.paddle1_y.to_be_bytes());
        msg.extend_from_slice(&state.paddle2_y.to_be_bytes());
        msg.extend_from_slice(&state.ball_x.to_be_bytes());
        msg.extend_from_slice(&state.ball_y.to_be_bytes());
        msg.push(state.score_p1);
        msg.push(state.score_p2);
        msg
    }

    /// Codifica fin de partida: [0x07] [0x00, 0x01] [Ganador: 1 o 2]
    pub fn encode_game_over(winner: u8) -> Vec<u8> {
        let mut msg = Vec::with_capacity(4);
        msg.extend_from_slice(&Self::encode_header(OpCode::GameOver, 1));
        msg.push(winner);
        msg
    }
}

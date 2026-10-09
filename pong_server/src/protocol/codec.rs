use super::packet::{Direction, GameStatePayload, OpCode};
use crate::net::socket::BerkeleyStream;
use std::time::{Duration, Instant};

pub struct Codec;
pub const MAX_REGISTER_PAYLOAD: usize = 280;
impl Codec {
    pub fn decode_header(buf: &[u8; 3]) -> (OpCode, u16) {
        (OpCode::from(buf[0]), u16::from_be_bytes([buf[1], buf[2]]))
    }
    /// TCP is a byte stream: one recv is never assumed to contain a frame.
    pub fn read_request(
        stream: &BerkeleyStream,
        registering: bool,
    ) -> Result<(OpCode, Vec<u8>), String> {
        let initial = registering.then(|| Instant::now() + Duration::from_secs(5));
        let mut header = [0; 3];
        stream
            .recv_exact(&mut header[..1], initial)
            .map_err(|e| e.to_string())?;
        let deadline = initial.or_else(|| Some(Instant::now() + Duration::from_secs(5)));
        stream
            .recv_exact(&mut header[1..], deadline)
            .map_err(|e| e.to_string())?;
        let (opcode, length) = Self::decode_header(&header);
        let valid = if registering {
            opcode == OpCode::RegisterReq && (2..=MAX_REGISTER_PAYLOAD).contains(&(length as usize))
        } else {
            opcode == OpCode::MoveInput && length == 1
        };
        if !valid {
            return Err(format!(
                "cabecera no permitida: opcode={} longitud={length}",
                header[0]
            ));
        }
        let mut payload = vec![0; length as usize];
        stream
            .recv_exact(&mut payload, deadline)
            .map_err(|e| e.to_string())?;
        Ok((opcode, payload))
    }
    pub fn encode_header(opcode: OpCode, length: u16) -> [u8; 3] {
        let bytes = length.to_be_bytes();
        [opcode as u8, bytes[0], bytes[1]]
    }
    pub fn decode_register_req(payload: &[u8]) -> Result<(String, String), String> {
        if payload.len() < 2 {
            return Err("registro incompleto".into());
        }
        let n = payload[0] as usize;
        if !(1..=24).contains(&n) || payload.len() < n + 2 {
            return Err("nickname: entre 1 y 24 bytes UTF-8".into());
        }
        let e = payload[n + 1] as usize;
        if !(3..=254).contains(&e) || payload.len() != n + e + 2 {
            return Err("longitud de correo inválida".into());
        }
        let nick = std::str::from_utf8(&payload[1..n + 1]).map_err(|_| "nickname no es UTF-8")?;
        let email = std::str::from_utf8(&payload[n + 2..]).map_err(|_| "correo no es UTF-8")?;
        if nick.trim() != nick || nick.chars().any(char::is_control) {
            return Err("nickname inválido".into());
        }
        let parts: Vec<_> = email.split('@').collect();
        if !email.is_ascii()
            || email.chars().any(char::is_whitespace)
            || email.chars().any(char::is_control)
            || parts.len() != 2
            || parts.iter().any(|p| p.is_empty())
            || !parts[1].contains('.')
            || parts[1].starts_with('.')
            || parts[1].ends_with('.')
        {
            return Err("correo inválido".into());
        }
        Ok((nick.to_owned(), email.to_owned()))
    }
    pub fn encode_register_resp(status: u8, id: u8) -> Vec<u8> {
        vec![2, 0, 2, status, id]
    }
    pub fn encode_wait_match() -> Vec<u8> {
        Self::encode_header(OpCode::WaitMatch, 0).to_vec()
    }
    pub fn encode_game_start(role: u8) -> Vec<u8> {
        vec![4, 0, 1, role]
    }
    pub fn decode_move_input(payload: &[u8]) -> Result<Direction, String> {
        match payload {
            [0] => Ok(Direction::Still),
            [1] => Ok(Direction::Up),
            [2] => Ok(Direction::Down),
            _ => Err("movimiento inválido".into()),
        }
    }
    pub fn encode_game_state(s: &GameStatePayload) -> Vec<u8> {
        let mut bytes = Self::encode_header(OpCode::GameState, 10).to_vec();
        for value in [s.paddle1_y, s.paddle2_y, s.ball_x, s.ball_y] {
            bytes.extend_from_slice(&value.to_be_bytes());
        }
        bytes.extend_from_slice(&[s.score_p1, s.score_p2]);
        bytes
    }
    pub fn encode_game_over(winner: u8) -> Vec<u8> {
        vec![7, 0, 1, winner]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registration_is_strict() {
        assert_eq!(
            Codec::decode_register_req(b"\x03Ana\x07a@b.com").unwrap(),
            ("Ana".into(), "a@b.com".into())
        );
        for invalid in [
            &b""[..],
            b"\0\0",
            b"\x03Ana\x07a@b.comX",
            b"\x01\xff\x07a@b.com",
            b"\x01A\x03bad",
            b"\x01\n\x07a@b.com",
        ] {
            assert!(Codec::decode_register_req(invalid).is_err(), "{invalid:?}");
        }
    }
    #[test]
    fn movement_rejects_unknown_and_trailing_bytes() {
        assert!(Codec::decode_move_input(&[2]).is_ok());
        for payload in [&[][..], &[3], &[1, 0]] {
            assert!(Codec::decode_move_input(payload).is_err());
        }
    }
    #[test]
    fn state_has_network_order_and_exact_length() {
        let s = GameStatePayload {
            paddle1_y: 0x123,
            paddle2_y: 0x145,
            ball_x: 0x267,
            ball_y: 0x189,
            score_p1: 4,
            score_p2: 2,
        };
        assert_eq!(
            Codec::encode_game_state(&s),
            [6, 0, 10, 1, 0x23, 1, 0x45, 2, 0x67, 1, 0x89, 4, 2]
        );
    }
}

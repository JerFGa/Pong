use crate::game::player::{Player, FINISHED, PLAYING};
use crate::game::state::GameState;
use crate::protocol::codec::Codec;
use crate::utils::logger::Logger;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub struct Room;
impl Room {
    pub fn run(id: u64, p1: Arc<Player>, p2: Arc<Player>) {
        let players = [&p1, &p2];
        Logger::info(&format!(
            "sala={id} inicio p1={} p2={}",
            p1.nickname, p2.nickname
        ));
        for (index, player) in players.iter().enumerate() {
            player.phase.store(PLAYING, Ordering::SeqCst);
            if player
                .stream
                .send_all(&Codec::encode_game_start(index as u8 + 1))
                .is_err()
            {
                player.disconnect();
            } else {
                Logger::response(&format!(
                    "sala={id} id={} GAME_START rol={}",
                    player.reservation.id,
                    index + 1
                ));
            }
        }
        let mut state = GameState::new();
        let tick = Duration::from_millis(25);
        let mut next_tick = Instant::now();
        let winner = loop {
            let active = (p1.is_connected(), p2.is_connected());
            if active != (true, true) {
                Logger::info(&format!(
                    "sala={id} fin por desconexión p1={} p2={}",
                    active.0, active.1
                ));
                break match active {
                    (true, false) => 1,
                    (false, true) => 2,
                    _ => 0,
                };
            }
            for (index, player) in players.iter().enumerate() {
                let direction = match player.direction.load(Ordering::SeqCst) {
                    1 => -1,
                    2 => 1,
                    _ => 0,
                };
                state.move_paddle(index as u8 + 1, direction);
            }
            let old_score = (state.score_p1, state.score_p2);
            let scored_winner = state.update_physics();
            if old_score != (state.score_p1, state.score_p2) {
                Logger::info(&format!(
                    "sala={id} gol marcador={}:{}",
                    state.score_p1, state.score_p2
                ));
            }
            let payload = state.to_payload();
            let bytes = Codec::encode_game_state(&payload);
            for player in players {
                if player.stream.send_all(&bytes).is_err() {
                    player.disconnect();
                } else {
                    Logger::response(&format!(
                        "sala={id} id={} GAME_STATE paletas={},{} pelota={},{} marcador={}:{}",
                        player.reservation.id,
                        payload.paddle1_y,
                        payload.paddle2_y,
                        payload.ball_x,
                        payload.ball_y,
                        payload.score_p1,
                        payload.score_p2
                    ));
                }
            }
            if let Some(winner) = scored_winner {
                break winner;
            }
            next_tick += tick;
            if let Some(delay) = next_tick.checked_duration_since(Instant::now()) {
                thread::sleep(delay);
            } else {
                next_tick = Instant::now();
            }
        };
        for player in players {
            player.phase.store(FINISHED, Ordering::SeqCst);
            if player.is_connected()
                && player
                    .stream
                    .send_all(&Codec::encode_game_over(winner))
                    .is_ok()
            {
                Logger::response(&format!(
                    "sala={id} id={} GAME_OVER ganador={winner}",
                    player.reservation.id
                ));
            }
            player.disconnect();
        }
        Logger::info(&format!("sala={id} cerrada ganador={winner}"));
    }
}

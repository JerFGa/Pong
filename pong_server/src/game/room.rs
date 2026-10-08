use crate::game::player::Player;
use crate::game::state::GameState;
use crate::protocol::codec::Codec;
use crate::protocol::packet::{Direction, OpCode};
use crate::utils::logger::Logger;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct Room;

impl Room {
    pub fn start_match(p1: Player, p2: Player) {
        thread::spawn(move || {
            let room_name = format!("Sala-[{} vs {}]", p1.nickname, p2.nickname);
            Logger::info(&format!("{}: Iniciando partida...", room_name));

            // 1. Notificar a cada jugador que inicia la partida y su respectivo rol
            let p1_start = Codec::encode_game_start(1);
            let p2_start = Codec::encode_game_start(2);

            if p1.stream.send_all(&p1_start).is_err() || p2.stream.send_all(&p2_start).is_err() {
                Logger::error(&format!("{}: Error notificando inicio a los clientes", room_name));
                return;
            }

            Logger::response(&format!("{}: MSG_GAME_START enviado a ambos jugadores", room_name));

            let state = Arc::new(std::sync::Mutex::new(GameState::new()));
            let is_running = Arc::new(AtomicBool::new(true));

            // Hilo de escucha para entradas de Jugador 1
            let state_p1 = Arc::clone(&state);
            let is_running_p1 = Arc::clone(&is_running);
            let p1_fd = p1.stream.fd;
            let room_p1 = room_name.clone();
            thread::spawn(move || {
                let mut header_buf = [0u8; 3];
                while is_running_p1.load(Ordering::Relaxed) {
                    let n = unsafe {
                        libc::recv(p1_fd, header_buf.as_mut_ptr() as *mut libc::c_void, 3, 0)
                    };
                    if n <= 0 { break; }

                    let (opcode, len) = Codec::decode_header(&header_buf);
                    let mut payload = vec![0u8; len as usize];
                    if len > 0 {
                        let _ = unsafe {
                            libc::recv(p1_fd, payload.as_mut_ptr() as *mut libc::c_void, len as usize, 0)
                        };
                    }

                    if opcode == OpCode::MoveInput {
                        let dir = Codec::decode_move_input(&payload);
                        let dir_val = match dir {
                            Direction::Up => -1,
                            Direction::Down => 1,
                            Direction::Still => 0,
                        };
                        if let Ok(mut g) = state_p1.lock() {
                            g.move_paddle(1, dir_val);
                        }
                    }
                }
                is_running_p1.store(false, Ordering::Relaxed);
                Logger::info(&format!("{}: Jugador 1 desconectado", room_p1));
            });

            // Hilo de escucha para entradas de Jugador 2
            let state_p2 = Arc::clone(&state);
            let is_running_p2 = Arc::clone(&is_running);
            let p2_fd = p2.stream.fd;
            let room_p2 = room_name.clone();
            thread::spawn(move || {
                let mut header_buf = [0u8; 3];
                while is_running_p2.load(Ordering::Relaxed) {
                    let n = unsafe {
                        libc::recv(p2_fd, header_buf.as_mut_ptr() as *mut libc::c_void, 3, 0)
                    };
                    if n <= 0 { break; }

                    let (opcode, len) = Codec::decode_header(&header_buf);
                    let mut payload = vec![0u8; len as usize];
                    if len > 0 {
                        let _ = unsafe {
                            libc::recv(p2_fd, payload.as_mut_ptr() as *mut libc::c_void, len as usize, 0)
                        };
                    }

                    if opcode == OpCode::MoveInput {
                        let dir = Codec::decode_move_input(&payload);
                        let dir_val = match dir {
                            Direction::Up => -1,
                            Direction::Down => 1,
                            Direction::Still => 0,
                        };
                        if let Ok(mut g) = state_p2.lock() {
                            g.move_paddle(2, dir_val);
                        }
                    }
                }
                is_running_p2.store(false, Ordering::Relaxed);
                Logger::info(&format!("{}: Jugador 2 desconectado", room_p2));
            });

            // Bucle principal de la física y sincronización del juego (40 FPS = 25ms por tick)
            let tick_rate = Duration::from_millis(25);
            let mut winner = None;

            while is_running.load(Ordering::Relaxed) {
                thread::sleep(tick_rate);

                let (game_payload, maybe_winner) = {
                    let mut g = state.lock().unwrap();
                    let w = g.update_physics();
                    (g.to_payload(), w)
                };

                // Transmitir estado del juego a ambos clientes
                let state_bytes = Codec::encode_game_state(&game_payload);
                if p1.stream.send_all(&state_bytes).is_err() || p2.stream.send_all(&state_bytes).is_err() {
                    Logger::error(&format!("{}: Error transmitiendo estado de juego", room_name));
                    break;
                }

                if let Some(w) = maybe_winner {
                    winner = Some(w);
                    break;
                }
            }

            // Fin de la partida
            if let Some(w) = winner {
                Logger::info(&format!("{}: Partida finalizada. Ganador: Jugador {}", room_name, w));
                let over_bytes = Codec::encode_game_over(w);
                let _ = p1.stream.send_all(&over_bytes);
                let _ = p2.stream.send_all(&over_bytes);
            }

            is_running.store(false, Ordering::Relaxed);
            Logger::info(&format!("{}: Sala cerrada y recursos liberados.", room_name));
        });
    }
}

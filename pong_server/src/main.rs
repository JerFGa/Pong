mod game;
mod net;
mod protocol;
mod utils;

use game::player::Player;
use game::room::Room;
use net::socket::BerkeleyListener;
use protocol::codec::Codec;
use protocol::packet::OpCode;
use utils::logger::Logger;

use std::env;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Uso: {} <PORT> <Log File>", args[0]);
        std::process::exit(1);
    }

    let port: u16 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Error: El puerto debe ser un número entero válido.");
            std::process::exit(1);
        }
    };

    let log_file = &args[2];

    // Inicializar el sistema de bitácora
    if let Err(e) = Logger::init(log_file) {
        eprintln!("Error al inicializar el archivo de log '{}': {}", log_file, e);
        std::process::exit(1);
    }

    Logger::info(&format!("Iniciando servidor de juego Pong en el puerto {}", port));
    Logger::info(&format!("Registrando eventos en el archivo '{}'", log_file));

    // Inicializar el socket con la API de Berkeley
    let listener = match BerkeleyListener::bind(port) {
        Ok(l) => l,
        Err(e) => {
            Logger::error(&format!("No se pudo enlazar el socket Berkeley: {}", e));
            std::process::exit(1);
        }
    };

    Logger::info("Servidor listo y a la escucha de jugadores...");

    // Cola de espera para emparejar jugadores de dos en dos
    let waiting_player: Arc<Mutex<Option<Player>>> = Arc::new(Mutex::new(None));
    let mut player_counter: u8 = 1;

    loop {
        match listener.accept() {
            Ok((stream, client_ip)) => {
                Logger::request(&format!("Nueva conexión entrante desde {}", client_ip));

                let waiting_queue = Arc::clone(&waiting_player);
                let current_id = player_counter;
                player_counter = player_counter.wrapping_add(1);

                // Procesar registro en un hilo para no bloquear el bucle de aceptación
                thread::spawn(move || {
                    let mut header_buf = [0u8; 3];
                    if stream.recv_exact(&mut header_buf).is_err() {
                        Logger::error("Error leyendo cabecera de registro");
                        return;
                    }

                    let (opcode, len) = Codec::decode_header(&header_buf);
                    if opcode != OpCode::RegisterReq {
                        Logger::error(&format!("OpCode inesperado durante el registro: {:?}", opcode));
                        return;
                    }

                    let mut payload = vec![0u8; len as usize];
                    if stream.recv_exact(&mut payload).is_err() {
                        Logger::error("Error leyendo payload de registro");
                        return;
                    }

                    let (nick, email) = match Codec::decode_register_req(&payload) {
                        Ok(res) => res,
                        Err(e) => {
                            Logger::error(&format!("Error al decodificar registro: {}", e));
                            return;
                        }
                    };

                    Logger::info(&format!(
                        "Jugador registrado exitosamente: Nickname='{}', Email='{}', ID={}",
                        nick, email, current_id
                    ));

                    // Confirmar registro al cliente
                    let resp = Codec::encode_register_resp(0, current_id);
                    if stream.send_all(&resp).is_err() {
                        Logger::error("Error enviando respuesta de registro al cliente");
                        return;
                    }
                    Logger::response(&format!("MSG_REGISTER_RESP enviado a {}", nick));

                    let new_player = Player::new(current_id, nick, email, stream);

                    // Lógica de emparejamiento (Matchmaking)
                    let mut queue = waiting_queue.lock().unwrap();
                    if let Some(rival) = queue.take() {
                        Logger::info(&format!(
                            "Emparejando a '{}' (P1) con '{}' (P2)",
                            rival.nickname, new_player.nickname
                        ));
                        // Iniciar la sala de juego en un nuevo hilo
                        Room::start_match(rival, new_player);
                    } else {
                        // Notificar que está en espera
                        let wait_msg = Codec::encode_wait_match();
                        let _ = new_player.stream.send_all(&wait_msg);
                        Logger::response(&format!("MSG_WAIT_MATCH enviado a {}", new_player.nickname));
                        *queue = Some(new_player);
                    }
                });
            }
            Err(e) => {
                Logger::error(&format!("Error al aceptar conexión: {}", e));
            }
        }
    }
}

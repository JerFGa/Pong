#[cfg(not(target_os = "linux"))]
compile_error!("PongServer utiliza Berkeley/Linux. En Windows, compila dentro de Ubuntu/WSL.");
mod game;
mod net;
mod protocol;
mod utils;

use game::player::{Player, Registry, Reservation, FINISHED, PLAYING};
use game::room::Room;
use net::socket::{BerkeleyListener, BerkeleyStream};
use protocol::codec::Codec;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use utils::logger::Logger;

static STOP: AtomicBool = AtomicBool::new(false);
extern "C" fn stop(_: libc::c_int) {
    STOP.store(true, Ordering::SeqCst);
}

#[derive(Default)]
struct Lobby {
    waiting: Mutex<Option<Weak<Player>>>,
    rooms: Mutex<Vec<JoinHandle<()>>>,
    next_room: AtomicU64,
}
impl Lobby {
    fn join(&self, player: &Arc<Player>) -> Result<(), String> {
        let mut waiting = self.waiting.lock().unwrap();
        if let Some(rival) = waiting
            .take()
            .and_then(|p| p.upgrade())
            .filter(|p| p.is_connected())
        {
            let id = self.next_room.fetch_add(1, Ordering::SeqCst) + 1;
            let player = Arc::clone(player);
            let mut rooms = self.rooms.lock().unwrap();
            reap(&mut rooms);
            rooms.push(thread::spawn(move || Room::run(id, rival, player)));
        } else {
            player
                .stream
                .send_all(&Codec::encode_wait_match())
                .map_err(|e| e.to_string())?;
            Logger::response(&format!("id={} WAIT_MATCH", player.reservation.id));
            *waiting = Some(Arc::downgrade(player));
        }
        Ok(())
    }
}
fn reap(handles: &mut Vec<JoinHandle<()>>) {
    let mut index = 0;
    while index < handles.len() {
        if handles[index].is_finished() {
            if handles.swap_remove(index).join().is_err() {
                Logger::error("hilo terminó inesperadamente");
            }
        } else {
            index += 1;
        }
    }
}
fn session(stream: Arc<BerkeleyStream>, reservation: Reservation, lobby: Arc<Lobby>) {
    let id = reservation.id;
    let registration = Codec::read_request(&stream, true)
        .and_then(|(_, payload)| Codec::decode_register_req(&payload));
    let (nick, email) = match registration {
        Ok(profile) => profile,
        Err(error) => {
            Logger::error(&format!("id={id} registro rechazado: {error}"));
            let _ = stream.send_all(&Codec::encode_register_resp(1, 0));
            Logger::response(&format!("id={id} REGISTER_RESP estado=1"));
            stream.shutdown();
            return;
        }
    };
    Logger::request(&format!(
        "id={id} REGISTER_REQ nickname={nick} email={email}"
    ));
    if !reservation.set_name(&nick) {
        let _ = stream.send_all(&Codec::encode_register_resp(2, 0));
        Logger::response(&format!("id={id} REGISTER_RESP estado=2 nickname ocupado"));
        stream.shutdown();
        return;
    }
    if stream
        .send_all(&Codec::encode_register_resp(0, id))
        .is_err()
    {
        stream.shutdown();
        return;
    }
    Logger::response(&format!("id={id} REGISTER_RESP estado=0"));
    let player = Arc::new(Player::new(reservation, nick, stream));
    if let Err(error) = lobby.join(&player) {
        Logger::error(&format!("id={id} emparejamiento: {error}"));
        player.disconnect();
        return;
    }
    let mut window = Instant::now();
    let mut messages = 0u16;
    while player.is_connected() && !STOP.load(Ordering::SeqCst) {
        let input = Codec::read_request(&player.stream, false)
            .and_then(|(_, payload)| Codec::decode_move_input(&payload));
        let direction = match input {
            Ok(direction) => direction,
            Err(error) => {
                if player.phase.load(Ordering::SeqCst) != FINISHED {
                    Logger::error(&format!("id={id} lectura: {error}"));
                }
                break;
            }
        };
        if window.elapsed() >= Duration::from_secs(1) {
            window = Instant::now();
            messages = 0;
        }
        messages += 1;
        if player.phase.load(Ordering::SeqCst) == FINISHED {
            continue; // Let the room send GAME_OVER before it shuts down the reader.
        }
        if messages > 240 || player.phase.load(Ordering::SeqCst) != PLAYING {
            if player.phase.load(Ordering::SeqCst) != FINISHED {
                Logger::error(&format!(
                    "id={id} movimiento fuera de fase o límite de 240 mensajes/s"
                ));
            }
            break;
        }
        player.direction.store(direction as u8, Ordering::SeqCst);
        Logger::request(&format!("id={id} MOVE_INPUT dirección={}", direction as u8));
    }
    player.disconnect();
    Logger::info(&format!("id={id} sesión finalizada"));
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err(format!("Uso: {} <PORT> <Log File>", args[0]));
    }
    let port = args[1]
        .parse::<u16>()
        .ok()
        .filter(|p| *p > 0)
        .ok_or("PORT debe estar entre 1 y 65535")?;
    Logger::init(&args[2]).map_err(|e| format!("no se pudo abrir la bitácora: {e}"))?;
    let listener =
        BerkeleyListener::bind(port).map_err(|e| format!("no se pudo escuchar en {port}: {e}"))?;
    // SAFETY: signal handlers only set an atomic flag; no allocation or locks.
    unsafe {
        libc::signal(libc::SIGINT, stop as *const () as libc::sighandler_t);
        libc::signal(libc::SIGTERM, stop as *const () as libc::sighandler_t);
    }
    let registry = Arc::new(Mutex::new(Registry::default()));
    let lobby = Arc::new(Lobby::default());
    let mut sessions = Vec::new();
    Logger::info(&format!(
        "PongServer listo 0.0.0.0:{port} TCP; 40 ticks/s; límite 255 conexiones"
    ));
    while !STOP.load(Ordering::SeqCst) {
        reap(&mut sessions);
        reap(&mut lobby.rooms.lock().unwrap());
        match listener.accept() {
            Ok(Some((stream, peer))) => {
                let stream = Arc::new(stream);
                if let Some(reservation) = Reservation::reserve(&registry, &stream) {
                    Logger::info(&format!("id={} conexión={peer}", reservation.id));
                    let lobby = Arc::clone(&lobby);
                    sessions.push(thread::spawn(move || session(stream, reservation, lobby)));
                } else {
                    let _ = stream.send_all(&Codec::encode_register_resp(3, 0));
                    Logger::response("REGISTER_RESP estado=3 servidor lleno");
                    stream.shutdown();
                }
            }
            Ok(None) => (),
            Err(error) => {
                Logger::error(&format!("accept: {error}"));
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
    registry.lock().unwrap().shutdown_all();
    for handle in sessions {
        let _ = handle.join();
    }
    for handle in lobby.rooms.lock().unwrap().drain(..) {
        let _ = handle.join();
    }
    Logger::info("Servidor detenido; sesiones y salas cerradas");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

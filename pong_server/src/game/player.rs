use crate::net::socket::BerkeleyStream;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, Weak};
pub const WAITING: u8 = 0;
pub const PLAYING: u8 = 1;
pub const FINISHED: u8 = 2;

#[derive(Default)]
pub struct Registry {
    entries: HashMap<u8, (String, Weak<BerkeleyStream>)>,
}
pub type SharedRegistry = Arc<Mutex<Registry>>;
/// Reservation is released on every exit path, including failed registration.
pub struct Reservation {
    pub id: u8,
    registry: SharedRegistry,
}
impl Reservation {
    pub fn reserve(registry: &SharedRegistry, stream: &Arc<BerkeleyStream>) -> Option<Self> {
        let mut r = registry.lock().unwrap();
        let id = (1..=255).find(|id| !r.entries.contains_key(id))?;
        r.entries
            .insert(id, (String::new(), Arc::downgrade(stream)));
        Some(Self {
            id,
            registry: Arc::clone(registry),
        })
    }
    pub fn set_name(&self, name: &str) -> bool {
        let mut r = self.registry.lock().unwrap();
        let key = name.to_lowercase();
        if r.entries.values().any(|(name, _)| name == &key) {
            return false;
        }
        r.entries.get_mut(&self.id).unwrap().0 = key;
        true
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        self.registry.lock().unwrap().entries.remove(&self.id);
    }
}
impl Registry {
    pub fn shutdown_all(&self) {
        for (_, stream) in self.entries.values() {
            if let Some(stream) = stream.upgrade() {
                stream.shutdown();
            }
        }
    }
}
pub struct Player {
    pub reservation: Reservation,
    pub nickname: String,
    pub stream: Arc<BerkeleyStream>,
    pub connected: AtomicBool,
    pub direction: AtomicU8,
    pub phase: AtomicU8,
}
impl Player {
    pub fn new(reservation: Reservation, nickname: String, stream: Arc<BerkeleyStream>) -> Self {
        Self {
            reservation,
            nickname,
            stream,
            connected: AtomicBool::new(true),
            direction: AtomicU8::new(0),
            phase: AtomicU8::new(WAITING),
        }
    }
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
    pub fn disconnect(&self) {
        self.connected.store(false, Ordering::SeqCst);
        self.direction.store(0, Ordering::SeqCst);
        self.stream.shutdown();
    }
}

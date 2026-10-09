use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
pub struct Logger {
    file: Mutex<File>,
}
static GLOBAL: OnceLock<Logger> = OnceLock::new();
impl Logger {
    pub fn init(path: &str) -> io::Result<()> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        GLOBAL
            .set(Logger {
                file: Mutex::new(file),
            })
            .map_err(|_| io::Error::other("logger ya inicializado"))
    }
    pub fn log(level: &str, message: &str) {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let line = format!(
            "[{}.{:03} UTC-epoch] [{level}] {message}\n",
            time.as_secs(),
            time.subsec_millis()
        );
        if let Some(logger) = GLOBAL.get() {
            let mut file = logger.file.lock().unwrap();
            let _ = io::stdout().lock().write_all(line.as_bytes());
            if let Err(error) = file.write_all(line.as_bytes()).and_then(|_| file.flush()) {
                let _ = writeln!(io::stderr().lock(), "Error escribiendo bitácora: {error}");
            }
        }
    }
    pub fn info(message: &str) {
        Self::log("INFO", message);
    }
    pub fn error(message: &str) {
        Self::log("ERROR", message);
    }
    pub fn request(message: &str) {
        Self::log("REQUEST", message);
    }
    pub fn response(message: &str) {
        Self::log("RESPONSE", message);
    }
}

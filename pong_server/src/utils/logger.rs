use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Logger {
    file: Mutex<File>,
}

static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

impl Logger {
    pub fn init(file_path: &str) -> std::io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(file_path)?;

        let _ = GLOBAL_LOGGER.set(Logger {
            file: Mutex::new(file),
        });
        Ok(())
    }

    pub fn log(level: &str, message: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let formatted = format!("[{}] [{}] {}\n", now, level, message);

        // Imprimir por consola
        print!("{}", formatted);

        // Escribir en el archivo de log
        if let Some(logger) = GLOBAL_LOGGER.get() {
            if let Ok(mut file) = logger.file.lock() {
                let _ = file.write_all(formatted.as_bytes());
                let _ = file.flush();
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

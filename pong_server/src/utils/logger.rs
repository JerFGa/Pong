use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Logger {
    file: Mutex<File>,
}

// 2. La instancia global segura para todo el servidor
static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

// 3. Los métodos públicos de la clase
impl Logger {
    /// Se llama UNA SOLA VEZ al iniciar el servidor en main()
    pub fn init(file_path: &str) -> std::io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true) // No sobreescribir, agregar al final
            .open(file_path)?;

        let _ = GLOBAL_LOGGER.set(Logger {
            file: Mutex::new(file),
        });
        Ok(())
    }

    /// Método base que imprime en terminal y escribe en disco con fecha
    pub fn log(level: &str, message: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Formato estándar de bitácora: [TIMESTAMP] [NIVEL] Mensaje
        let formatted = format!("[{}] [{}] {}\n", now, level, message);

        // 1. Imprimir por consola
        print!("{}", formatted);

        // 2. Escribir en el archivo de log usando el semáforo Mutex
        if let Some(logger) = GLOBAL_LOGGER.get() {
            if let Ok(mut file) = logger.file.lock() {
                let _ = file.write_all(formatted.as_bytes());
                let _ = file.flush(); // Forzar escritura inmediata al disco
            }
        }
    }

    // Métodos de ayuda
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
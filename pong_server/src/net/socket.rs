use libc::*;
use std::mem;

/// Socket en modo servidor (Escucha conexiones usando Berkeley API)
pub struct BerkeleyListener {
    fd: c_int,
}

/// Socket de conexión bidireccional con un cliente
pub struct BerkeleyStream {
    pub fd: c_int,
}

impl BerkeleyListener {
    pub fn bind(port: u16) -> Result<Self, String> {
        unsafe {
            // 1. Crear socket
            let fd = socket(AF_INET, SOCK_STREAM, 0);
            if fd < 0 {
                return Err("Fallo al crear socket con la API de Berkeley".to_string());
            }

            // SO_REUSEADDR para poder reiniciar el servidor rápidamente
            let opt: c_int = 1;
            setsockopt(
                fd,
                SOL_SOCKET,
                SO_REUSEADDR,
                &opt as *const _ as *const c_void,
                mem::size_of_val(&opt) as socklen_t,
            );

            // 2. Configurar sockaddr_in
            let mut addr: sockaddr_in = mem::zeroed();
            addr.sin_family = AF_INET as sa_family_t;
            addr.sin_addr.s_addr = INADDR_ANY.to_be();
            addr.sin_port = port.to_be();

            // 3. Enlazar (bind)
            if bind(
                fd,
                &addr as *const _ as *const sockaddr,
                mem::size_of::<sockaddr_in>() as socklen_t,
            ) < 0
            {
                close(fd);
                return Err(format!("Fallo al asociar el puerto {} (bind)", port));
            }

            // 4. Escuchar (listen)
            if listen(fd, 32) < 0 {
                close(fd);
                return Err("Fallo al poner el socket en modo listen".to_string());
            }

            Ok(Self { fd })
        }
    }

    pub fn accept(&self) -> Result<(BerkeleyStream, String), String> {
        unsafe {
            let mut client_addr: sockaddr_in = mem::zeroed();
            let mut addr_len = mem::size_of::<sockaddr_in>() as socklen_t;

            let client_fd = accept(
                self.fd,
                &mut client_addr as *mut _ as *mut sockaddr,
                &mut addr_len,
            );

            if client_fd < 0 {
                return Err("Error al aceptar conexión entrante".to_string());
            }

            // Obtener IP del cliente en texto para el log
            let ip_bytes = client_addr.sin_addr.s_addr.to_ne_bytes();
            let ip_str = format!("{}.{}.{}.{}", ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);

            Ok((BerkeleyStream { fd: client_fd }, ip_str))
        }
    }
}

impl BerkeleyStream {
    pub fn send_all(&self, data: &[u8]) -> Result<(), String> {
        let mut total_sent = 0;
        while total_sent < data.len() {
            let sent = unsafe {
                send(
                    self.fd,
                    data[total_sent..].as_ptr() as *const c_void,
                    data.len() - total_sent,
                    0,
                )
            };
            if sent < 0 {
                return Err("Error de socket al enviar datos".to_string());
            }
            total_sent += sent as usize;
        }
        Ok(())
    }

    pub fn recv_exact(&self, buf: &mut [u8]) -> Result<(), String> {
        let mut total_read = 0;
        while total_read < buf.len() {
            let n = unsafe {
                recv(
                    self.fd,
                    buf[total_read..].as_mut_ptr() as *mut c_void,
                    buf.len() - total_read,
                    0,
                )
            };
            if n <= 0 {
                return Err("Conexión cerrada por el cliente o error de lectura".to_string());
            }
            total_read += n as usize;
        }
        Ok(())
    }
}

impl Drop for BerkeleyListener {
    fn drop(&mut self) {
        unsafe {
            if self.fd >= 0 {
                close(self.fd);
            }
        }
    }
}

impl Drop for BerkeleyStream {
    fn drop(&mut self) {
        unsafe {
            if self.fd >= 0 {
                close(self.fd);
            }
        }
    }
}

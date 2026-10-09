//! Linux Berkeley sockets. Descriptors never escape this module.
use std::io;
use std::mem;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::time::{Duration, Instant};

pub struct BerkeleyListener(OwnedFd);
pub struct BerkeleyStream(OwnedFd);

fn last_error() -> io::Error {
    io::Error::last_os_error()
}
fn wait(fd: i32, events: i16, deadline: Option<Instant>) -> io::Result<()> {
    loop {
        let timeout = match deadline {
            Some(end) => end
                .checked_duration_since(Instant::now())
                .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "plazo de socket agotado"))?
                .as_millis()
                .clamp(1, i32::MAX as u128) as i32,
            None => -1,
        };
        let mut pfd = libc::pollfd {
            fd,
            events,
            revents: 0,
        };
        // SAFETY: pfd is valid for one element during poll.
        let result = unsafe { libc::poll(&mut pfd, 1, timeout) };
        if result > 0 {
            if pfd.revents & libc::POLLNVAL != 0 {
                return Err(io::Error::other("descriptor inválido"));
            }
            return Ok(()); // recv/send report EOF and errors, including POLLHUP.
        }
        if result == 0 {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "plazo de socket agotado",
            ));
        }
        let error = last_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}
impl BerkeleyListener {
    pub fn bind(port: u16) -> io::Result<Self> {
        // SAFETY: socket returns a new descriptor, owned exactly once below.
        let raw = unsafe {
            libc::socket(
                libc::AF_INET,
                libc::SOCK_STREAM | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                0,
            )
        };
        if raw < 0 {
            return Err(last_error());
        }
        let fd = unsafe { OwnedFd::from_raw_fd(raw) };
        let one: libc::c_int = 1;
        let result = unsafe {
            libc::setsockopt(
                raw,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                &one as *const _ as *const libc::c_void,
                mem::size_of_val(&one) as libc::socklen_t,
            )
        };
        if result < 0 {
            return Err(last_error());
        }
        let mut address: libc::sockaddr_in = unsafe { mem::zeroed() };
        address.sin_family = libc::AF_INET as libc::sa_family_t;
        address.sin_port = port.to_be();
        address.sin_addr.s_addr = libc::INADDR_ANY;
        let result = unsafe {
            libc::bind(
                raw,
                &address as *const _ as *const libc::sockaddr,
                mem::size_of_val(&address) as libc::socklen_t,
            )
        };
        if result < 0 {
            return Err(last_error());
        }
        if unsafe { libc::listen(raw, 128) } < 0 {
            return Err(last_error());
        }
        Ok(Self(fd))
    }
    pub fn accept(&self) -> io::Result<Option<(BerkeleyStream, String)>> {
        match wait(
            self.0.as_raw_fd(),
            libc::POLLIN,
            Some(Instant::now() + Duration::from_millis(200)),
        ) {
            Err(e) if e.kind() == io::ErrorKind::TimedOut => return Ok(None),
            result => result?,
        }
        let mut address: libc::sockaddr_in = unsafe { mem::zeroed() };
        let mut length = mem::size_of_val(&address) as libc::socklen_t;
        let raw = unsafe {
            libc::accept4(
                self.0.as_raw_fd(),
                &mut address as *mut _ as *mut libc::sockaddr,
                &mut length,
                libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
            )
        };
        if raw < 0 {
            let error = last_error();
            return if matches!(
                error.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
            ) {
                Ok(None)
            } else {
                Err(error)
            };
        }
        let fd = unsafe { OwnedFd::from_raw_fd(raw) };
        let one: libc::c_int = 1;
        let result = unsafe {
            libc::setsockopt(
                raw,
                libc::IPPROTO_TCP,
                libc::TCP_NODELAY,
                &one as *const _ as *const libc::c_void,
                mem::size_of_val(&one) as libc::socklen_t,
            )
        };
        if result < 0 {
            return Err(last_error());
        }
        let ip = std::net::Ipv4Addr::from(address.sin_addr.s_addr.to_ne_bytes());
        // Bound dead-peer detection, including idle players in the lobby.
        for (level, option, value) in [
            (libc::SOL_SOCKET, libc::SO_KEEPALIVE, 1_i32),
            (libc::SOL_SOCKET, libc::SO_SNDBUF, 8192),
            (libc::IPPROTO_TCP, libc::TCP_KEEPIDLE, 30),
            (libc::IPPROTO_TCP, libc::TCP_KEEPINTVL, 10),
            (libc::IPPROTO_TCP, libc::TCP_KEEPCNT, 3),
            (libc::IPPROTO_TCP, libc::TCP_USER_TIMEOUT, 10000),
        ] {
            let result = unsafe {
                libc::setsockopt(
                    raw,
                    level,
                    option,
                    &value as *const _ as *const libc::c_void,
                    mem::size_of_val(&value) as libc::socklen_t,
                )
            };
            if result < 0 {
                return Err(last_error());
            }
        }
        Ok(Some((
            BerkeleyStream(fd),
            format!("{}:{}", ip, u16::from_be(address.sin_port)),
        )))
    }
}
impl BerkeleyStream {
    /// Bounded full write. The application assigns one writer per connection.
    pub fn send_all(&self, data: &[u8]) -> io::Result<()> {
        let deadline = Some(Instant::now() + Duration::from_secs(2));
        let mut offset = 0;
        while offset < data.len() {
            wait(self.0.as_raw_fd(), libc::POLLOUT, deadline)?;
            let n = unsafe {
                libc::send(
                    self.0.as_raw_fd(),
                    data[offset..].as_ptr().cast(),
                    data.len() - offset,
                    libc::MSG_NOSIGNAL | libc::MSG_DONTWAIT,
                )
            };
            if n > 0 {
                offset += n as usize;
            } else if n == 0 {
                return Err(io::Error::new(io::ErrorKind::WriteZero, "envío vacío"));
            } else {
                let error = last_error();
                if !matches!(
                    error.kind(),
                    io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
                ) {
                    return Err(error);
                }
            }
        }
        Ok(())
    }
    pub fn recv_exact(&self, data: &mut [u8], deadline: Option<Instant>) -> io::Result<()> {
        let mut offset = 0;
        while offset < data.len() {
            wait(self.0.as_raw_fd(), libc::POLLIN, deadline)?;
            let n = unsafe {
                libc::recv(
                    self.0.as_raw_fd(),
                    data[offset..].as_mut_ptr().cast(),
                    data.len() - offset,
                    libc::MSG_DONTWAIT,
                )
            };
            if n > 0 {
                offset += n as usize;
            } else if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "cliente desconectado",
                ));
            } else {
                let error = last_error();
                if !matches!(
                    error.kind(),
                    io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
                ) {
                    return Err(error);
                }
            }
        }
        Ok(())
    }
    pub fn shutdown(&self) {
        // Wakes readers; OwnedFd closes only when the last owning Arc is dropped.
        unsafe {
            libc::shutdown(self.0.as_raw_fd(), libc::SHUT_RDWR);
        }
    }
}

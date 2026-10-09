"""Only this service handles sockets. The UI consumes a thread-safe event queue."""
import queue
import socket
import threading
import time
from protocol import register_packet, move_packet, decode_header, decode_payload


class NetworkService:
    def __init__(self, host, port):
        self.host, self.port = host, port
        self.events = queue.Queue(maxsize=512)
        self.sock = None
        self.thread = None
        self.cancelled = threading.Event()

    def connect(self, nickname, email):
        packet = register_packet(nickname, email)
        self.thread = threading.Thread(target=self._run, args=(packet,), daemon=True)
        self.thread.start()

    def _publish(self, kind, value=None):
        while not self.cancelled.is_set():
            try:
                self.events.put((kind, value), timeout=0.1)
                return
            except queue.Full:
                continue

    def _recv_exact(self, sock, length, deadline=None):
        data = bytearray()
        while len(data) < length:
            if self.cancelled.is_set():
                raise ConnectionAbortedError("Conexión cancelada")
            if deadline is not None and time.monotonic() >= deadline:
                raise TimeoutError("El servidor dejó una respuesta incompleta")
            try:
                chunk = sock.recv(length - len(data))
            except socket.timeout:
                continue
            if not chunk:
                raise EOFError("El servidor cerró la conexión")
            data.extend(chunk)
        return bytes(data)

    def _run(self, packet):
        sock = None
        try:
            sock = socket.create_connection((self.host, self.port), timeout=5)
            if self.cancelled.is_set():
                return
            self.sock = sock
            sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
            sock.settimeout(0.25)
            sock.sendall(packet)
            registered = False
            playing = False
            while not self.cancelled.is_set():
                initial_deadline = time.monotonic() + 5 if playing else (None if registered else time.monotonic() + 7)
                first = self._recv_exact(sock, 1, initial_deadline)
                deadline = time.monotonic() + 5
                header = first + self._recv_exact(sock, 2, deadline)
                opcode, length = decode_header(header)
                payload = self._recv_exact(sock, length, deadline)
                self._publish("packet", (opcode, decode_payload(opcode, payload)))
                if opcode == 2:
                    registered = True
                if opcode == 4:
                    playing = True
                if opcode == 7 or (opcode == 2 and payload[0] != 0):
                    break
        except (OSError, EOFError, ValueError) as error:
            self._publish("error", str(error))
        finally:
            if sock:
                sock.close()
            self.sock = None
            self._publish("closed")

    def send_move(self, direction):
        sock = self.sock
        if sock and not self.cancelled.is_set():
            try:
                sock.sendall(move_packet(direction))
            except OSError as error:
                self.close()
                return str(error)
        return None

    def close(self):
        self.cancelled.set()
        sock = self.sock
        if sock:
            try:
                sock.shutdown(socket.SHUT_RDWR)
            except OSError:
                pass
            sock.close()
        if self.thread and self.thread is not threading.current_thread():
            self.thread.join(timeout=0.3)

import socket
import struct
import threading

class NetworkService:
    OP_REGISTER_REQ  = 0x01
    OP_REGISTER_RESP = 0x02
    OP_WAIT_MATCH    = 0x03
    OP_GAME_START    = 0x04
    OP_MOVE_INPUT    = 0x05
    OP_GAME_STATE    = 0x06
    OP_GAME_OVER     = 0x07

    def __init__(self, host, port, model):
        self.host = host
        self.port = port
        self.model = model
        self.sock = None
        self.running = False
        self.thread = None

    def connect(self, nickname: str, email: str) -> bool:
        try:
            self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.sock.connect((self.host, self.port))
            self.running = True

            # Enviar petición de registro binaria
            nick_bytes = nickname.encode("utf-8")
            email_bytes = email.encode("utf-8")
            payload = struct.pack(f"!B{len(nick_bytes)}sB{len(email_bytes)}s", 
                                  len(nick_bytes), nick_bytes, 
                                  len(email_bytes), email_bytes)
            
            header = struct.pack("!BH", self.OP_REGISTER_REQ, len(payload))
            self.sock.sendall(header + payload)

            # Iniciar hilo de recepción
            self.thread = threading.Thread(target=self._listen_loop, daemon=True)
            self.thread.start()
            return True
        except Exception as e:
            print(f"[NetworkService] Error al conectar: {e}")
            self.model.status_message = f"Error al conectar con el servidor: {e}"
            return False

    def send_move(self, direction: int):
        """direction: 0=quieto, 1=arriba, 2=abajo"""
        if not self.running or not self.sock:
            return
        try:
            header = struct.pack("!BH", self.OP_MOVE_INPUT, 1)
            payload = struct.pack("!B", direction)
            self.sock.sendall(header + payload)
        except Exception as e:
            print(f"[NetworkService] Error enviando movimiento: {e}")

    def _recv_exact(self, n: int) -> bytes:
        data = bytearray()
        while len(data) < n:
            packet = self.sock.recv(n - len(data))
            if not packet:
                return None
            data.extend(packet)
        return bytes(data)

    def _listen_loop(self):
        while self.running:
            try:
                # 1. Leer cabecera de 3 bytes
                header_bytes = self._recv_exact(3)
                if not header_bytes:
                    break

                opcode, length = struct.unpack("!BH", header_bytes)

                # 2. Leer payload si existe
                payload = b""
                if length > 0:
                    payload = self._recv_exact(length)
                    if not payload:
                        break

                # 3. Procesar según OpCode
                if opcode == self.OP_REGISTER_RESP:
                    status, player_id = struct.unpack("!BB", payload)
                    print(f"[Network] Registro confirmado. ID={player_id}, Status={status}")

                elif opcode == self.OP_WAIT_MATCH:
                    self.model.status = "WAITING"
                    self.model.status_message = "En cola: Esperando a que se conecte tu rival..."

                elif opcode == self.OP_GAME_START:
                    role, = struct.unpack("!B", payload)
                    self.model.set_game_start(role)
                    print(f"[Network] Partida iniciada como Jugador {role}")

                elif opcode == self.OP_GAME_STATE:
                    p1_y, p2_y, bx, by, s1, s2 = struct.unpack("!HHHHBB", payload)
                    self.model.update_state(p1_y, p2_y, bx, by, s1, s2)

                elif opcode == self.OP_GAME_OVER:
                    winner, = struct.unpack("!B", payload)
                    self.model.set_game_over(winner)
                    print(f"[Network] Fin de partida. Ganador: Jugador {winner}")

            except Exception as e:
                print(f"[NetworkService] Error en bucle de red: {e}")
                break

        self.running = False
        print("[NetworkService] Conexión cerrada.")

    def close(self):
        self.running = False
        if self.sock:
            try:
                self.sock.close()
            except:
                pass

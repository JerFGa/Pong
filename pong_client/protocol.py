"""MyAppGameProtocol: three-byte header, strict lengths, network byte order."""
import struct
import unicodedata

REGISTER_REQ, REGISTER_RESP, WAIT_MATCH, GAME_START, MOVE_INPUT, GAME_STATE, GAME_OVER = range(1, 8)
SERVER_LENGTHS = {REGISTER_RESP: 2, WAIT_MATCH: 0, GAME_START: 1, GAME_STATE: 10, GAME_OVER: 1}


def validate_profile(nickname, email):
    nick, mail = nickname.encode("utf-8"), email.encode("utf-8")
    if not 1 <= len(nick) <= 24 or nickname.strip() != nickname or any(unicodedata.category(c) == "Cc" for c in nickname):
        raise ValueError("El apodo debe ocupar entre 1 y 24 bytes, sin espacios en los extremos.")
    parts = email.split("@")
    if (not 3 <= len(mail) <= 254 or not email.isascii()
            or any(c.isspace() or unicodedata.category(c) == "Cc" for c in email)
            or len(parts) != 2 or not all(parts) or "." not in parts[1]
            or parts[1].startswith(".") or parts[1].endswith(".")):
        raise ValueError("Escribe un correo válido, por ejemplo ana@eafit.edu.co.")
    return nick, mail


def register_packet(nickname, email):
    nick, mail = validate_profile(nickname, email)
    payload = bytes([len(nick)]) + nick + bytes([len(mail)]) + mail
    return struct.pack("!BH", REGISTER_REQ, len(payload)) + payload


def move_packet(direction):
    if direction not in (0, 1, 2):
        raise ValueError("Dirección inválida")
    return struct.pack("!BHB", MOVE_INPUT, 1, direction)


def decode_header(header):
    if len(header) != 3:
        raise ValueError("Cabecera incompleta")
    opcode, length = struct.unpack("!BH", header)
    if SERVER_LENGTHS.get(opcode) != length:
        raise ValueError("Respuesta desconocida o longitud incorrecta")
    return opcode, length


def decode_payload(opcode, payload):
    if SERVER_LENGTHS.get(opcode) != len(payload):
        raise ValueError("Carga útil incorrecta")
    if opcode == REGISTER_RESP:
        status, player_id = payload
        if status not in range(4) or (status == 0) != (player_id != 0):
            raise ValueError("Respuesta de registro inválida")
        return status, player_id
    if opcode == WAIT_MATCH:
        return ()
    if opcode in (GAME_START, GAME_OVER):
        value = payload[0]
        if value not in ((1, 2) if opcode == GAME_START else (0, 1, 2)):
            raise ValueError("Rol o ganador inválido")
        return (value,)
    values = struct.unpack("!HHHHBB", payload)
    p1, p2, bx, by, s1, s2 = values
    if p1 > 510 or p2 > 510 or bx > 800 or by > 588 or s1 > 5 or s2 > 5:
        raise ValueError("Estado de juego fuera de rango")
    return values

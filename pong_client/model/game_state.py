"""Passive state, updated exclusively by the UI/controller thread."""
from protocol import REGISTER_RESP, WAIT_MATCH, GAME_START, GAME_STATE, GAME_OVER

class GameStateModel:
    def __init__(self):
        self.paddle1_y = self.paddle2_y = 255
        self.ball_x, self.ball_y = 394, 294
        self.score_p1 = self.score_p2 = 0
        self.role = self.player_id = self.winner = None
        self.status = "REGISTER"
        self.status_message = "Crea tu perfil y entra a la cancha."

    def apply(self, opcode, values):
        if opcode == REGISTER_RESP and self.status == "CONNECTING":
            status, self.player_id = values
            if status:
                self.fail({1: "Perfil inválido.", 2: "Ese apodo ya está jugando. Elige otro.",
                           3: "El servidor está lleno. Inténtalo más tarde."}[status])
            else:
                self.status = "REGISTERED"
                self.status_message = "Perfil aceptado. Buscando rival..."
        elif opcode == WAIT_MATCH and self.status == "REGISTERED":
            self.status = "WAITING"
            self.status_message = "Esperando a otro jugador..."
        elif opcode == GAME_START and self.status in ("REGISTERED", "WAITING"):
            self.role = values[0]
            self.status = "PLAYING"
            self.status_message = f"Eres el jugador {self.role}"
        elif opcode == GAME_STATE and self.status == "PLAYING":
            (self.paddle1_y, self.paddle2_y, self.ball_x, self.ball_y,
             self.score_p1, self.score_p2) = values
        elif opcode == GAME_OVER and self.status == "PLAYING":
            self.winner = values[0]
            self.status = "GAME_OVER"
            if self.winner == 0:
                self.status_message = "Partida cancelada."
            elif self.winner == self.role:
                suffix = " Tu rival se desconectó." if max(self.score_p1, self.score_p2) < 5 else ""
                self.status_message = "¡Victoria!" + suffix
            else:
                self.status_message = "Tu rival ganó esta partida."
        else:
            raise ValueError("El servidor envió un mensaje fuera de secuencia")

    def fail(self, message):
        if self.status not in ("GAME_OVER", "ERROR"):
            self.status = "ERROR"
            self.status_message = message

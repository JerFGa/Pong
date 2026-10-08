"""Modelo de datos del juego Pong."""

class GameStateModel:
    def __init__(self):
        self.paddle1_y = 250
        self.paddle2_y = 250
        self.ball_x = 400
        self.ball_y = 300
        self.score_p1 = 0
        self.score_p2 = 0
        self.role = None          # 1 para Jugador 1 (Izq), 2 para Jugador 2 (Der)
        self.status = "CONNECTING"# "CONNECTING", "WAITING", "PLAYING", "GAME_OVER"
        self.winner = None
        self.status_message = "Conectando al servidor..."

    def update_state(self, p1_y, p2_y, ball_x, ball_y, s1, s2):
        self.paddle1_y = p1_y
        self.paddle2_y = p2_y
        self.ball_x = ball_x
        self.ball_y = ball_y
        self.score_p1 = s1
        self.score_p2 = s2

    def set_game_start(self, role):
        self.role = role
        self.status = "PLAYING"
        self.status_message = f"¡Partida iniciada! Eres el Jugador {role}"

    def set_game_over(self, winner):
        self.status = "GAME_OVER"
        self.winner = winner
        if winner == self.role:
            self.status_message = "¡VICTORIA! Has ganado la partida"
        else:
            self.status_message = "DERROTA. Tu rival ha ganado la partida"

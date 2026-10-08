import pygame

class GameView:
    WIDTH = 800
    HEIGHT = 600
    PADDLE_WIDTH = 15
    PADDLE_HEIGHT = 90
    BALL_SIZE = 12

    # Colores
    COLOR_BG = (20, 24, 30)
    COLOR_WHITE = (240, 240, 240)
    COLOR_ACCENT = (50, 180, 240)
    COLOR_TEXT = (200, 200, 200)

    def __init__(self):
        pygame.init()
        pygame.display.set_caption("Pong Online - Redes y Protocolos")
        self.screen = pygame.display.set_mode((self.WIDTH, self.HEIGHT))
        self.font_large = pygame.font.SysFont("Arial", 42, bold=True)
        self.font_mid = pygame.font.SysFont("Arial", 26)
        self.font_small = pygame.font.SysFont("Arial", 18)

    def render(self, model):
        self.screen.fill(self.COLOR_BG)

        if model.status == "PLAYING":
            self._render_playing(model)
        else:
            self._render_overlay(model)

        pygame.display.flip()

    def _render_playing(self, model):
        # Línea central punteada
        for y in range(0, self.HEIGHT, 30):
            pygame.draw.rect(self.screen, (60, 60, 80), (self.WIDTH // 2 - 2, y, 4, 15))

        # Marcador
        score_txt = f"{model.score_p1}   :   {model.score_p2}"
        surf_score = self.font_large.render(score_txt, True, self.COLOR_WHITE)
        self.screen.blit(surf_score, (self.WIDTH // 2 - surf_score.get_width() // 2, 20))

        # Paleta Jugador 1 (Izquierda)
        p1_color = self.COLOR_ACCENT if model.role == 1 else self.COLOR_WHITE
        pygame.draw.rect(self.screen, p1_color, (30, model.paddle1_y, self.PADDLE_WIDTH, self.PADDLE_HEIGHT), border_radius=4)

        # Paleta Jugador 2 (Derecha)
        p2_color = self.COLOR_ACCENT if model.role == 2 else self.COLOR_WHITE
        p2_x = self.WIDTH - 30 - self.PADDLE_WIDTH
        pygame.draw.rect(self.screen, p2_color, (p2_x, model.paddle2_y, self.PADDLE_WIDTH, self.PADDLE_HEIGHT), border_radius=4)

        # Pelota
        pygame.draw.rect(self.screen, (255, 220, 80), (model.ball_x, model.ball_y, self.BALL_SIZE, self.BALL_SIZE), border_radius=2)

        # Indicador de rol
        role_label = f"Tú eres: Jugador {model.role} ({'Izquierda' if model.role == 1 else 'Derecha'})"
        surf_role = self.font_small.render(role_label, True, self.COLOR_TEXT)
        self.screen.blit(surf_role, (20, self.HEIGHT - 30))

    def _render_overlay(self, model):
        msg = self.font_mid.render(model.status_message, True, self.COLOR_WHITE)
        rect = msg.get_rect(center=(self.WIDTH // 2, self.HEIGHT // 2))
        self.screen.blit(msg, rect)

        hint = self.font_small.render("Control: Flecha Arriba (W) / Flecha Abajo (S)", True, (120, 120, 140))
        hint_rect = hint.get_rect(center=(self.WIDTH // 2, self.HEIGHT // 2 + 50))
        self.screen.blit(hint, hint_rect)

    def close(self):
        pygame.quit()

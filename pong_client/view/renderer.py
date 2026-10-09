import pygame

class GameView:
    WIDTH, HEIGHT = 800, 600
    BG = (12, 19, 31)
    PANEL = (22, 33, 49)
    TEXT = (231, 239, 247)
    MUTED = (151, 169, 190)
    ACCENT = (62, 220, 205)
    GOLD = (255, 202, 104)

    def __init__(self):
        pygame.init()
        pygame.display.set_caption("PONG / Online")
        self.screen = pygame.display.set_mode((self.WIDTH, self.HEIGHT))
        self.large = pygame.font.SysFont("Arial", 52, bold=True)
        self.mid = pygame.font.SysFont("Arial", 24, bold=True)
        self.small = pygame.font.SysFont("Arial", 18)
        self.field_rects = [pygame.Rect(235, 174 + i * 64, 390, 40) for i in range(4)]
        self.play_button = pygame.Rect(235, 468, 390, 46)

    def text(self, value, pos, font=None, color=None, center=False):
        surface = (font or self.small).render(value, True, color or self.TEXT)
        rect = surface.get_rect(center=pos) if center else surface.get_rect(topleft=pos)
        self.screen.blit(surface, rect)

    def wrapped(self, value, y, color=None, width=640):
        # Break long unspaced errors too, so every response stays inside the window.
        lines, line = [], ""
        for char in value:
            if char == "\n" or self.small.size(line + char)[0] > width:
                lines.append(line)
                line = "" if char == "\n" else char
            else:
                line += char
        lines.append(line)
        for index, line in enumerate(lines[:4]):
            self.text(line, (400, y + index * 24), color=color, center=True)

    def render(self, model, fields=None, focus=0, error=""):
        self.screen.fill(self.BG)
        if model.status == "REGISTER":
            self._registration(fields or ["127.0.0.1", "8080", "", ""], focus, error)
        elif model.status == "PLAYING":
            self._court(model)
        else:
            self._overlay(model)
        pygame.display.flip()

    def _registration(self, fields, focus, error):
        self.text("PONG", (400, 58), self.large, center=True)
        self.text("ONLINE  /  DOS JUGADORES  /  PRIMERO A 5", (400, 106), color=self.ACCENT, center=True)
        labels = ["Servidor", "Puerto", "Apodo", "Correo"]
        for index, (rect, label, value) in enumerate(zip(self.field_rects, labels, fields)):
            self.text(label, (115, rect.y + 10), color=self.MUTED)
            pygame.draw.rect(self.screen, self.PANEL, rect, border_radius=7)
            pygame.draw.rect(self.screen, self.ACCENT if index == focus else (46, 63, 81), rect, width=2, border_radius=7)
            shown = value
            while self.small.size(shown)[0] > rect.width - 28:
                shown = shown[1:]
            self.text(shown, (rect.x + 12, rect.y + 9))
            if index == focus and pygame.time.get_ticks() % 1000 < 500:
                x = rect.x + 14 + self.small.size(shown)[0]
                pygame.draw.line(self.screen, self.ACCENT, (x, rect.y + 10), (x, rect.y + 29), 2)
        pygame.draw.rect(self.screen, self.ACCENT, self.play_button, border_radius=8)
        self.text("ENTRAR A JUGAR", self.play_button.center, self.mid, self.BG, center=True)
        if error:
            self.wrapped(error, 428, self.GOLD, 700)
        self.text("Tab: siguiente campo   ·   Enter: conectar   ·   Esc: salir", (400, 555), color=self.MUTED, center=True)

    def _court(self, model):
        for y in range(10, 600, 28):
            pygame.draw.rect(self.screen, (41, 58, 75), (398, y, 3, 12))
        self.text(f"{model.score_p1}      {model.score_p2}", (400, 50), self.large, center=True)
        for role, x, y in [(1, 30, model.paddle1_y), (2, 755, model.paddle2_y)]:
            color = self.ACCENT if model.role == role else self.TEXT
            pygame.draw.rect(self.screen, color, (x, y, 15, 90), border_radius=5)
        pygame.draw.rect(self.screen, self.GOLD, (model.ball_x, model.ball_y, 12, 12), border_radius=3)
        side = "IZQUIERDA" if model.role == 1 else "DERECHA"
        self.text(f"TU LADO: {side}", (90, 566), color=self.ACCENT)
        self.text("W / S o ↑ / ↓    ·    Esc: salir", (480, 566), color=self.MUTED)

    def _overlay(self, model):
        self.text("PONG", (400, 120), self.large, center=True)
        titles = {"CONNECTING": "CONECTANDO", "REGISTERED": "BUSCANDO RIVAL",
                  "WAITING": "SALA DE ESPERA", "ERROR": "NO SE PUDO CONTINUAR",
                  "GAME_OVER": "PARTIDA TERMINADA"}
        self.text(titles.get(model.status, "PONG ONLINE"), (400, 225), self.mid, self.ACCENT, center=True)
        self.wrapped(model.status_message, 295, self.GOLD if model.status == "ERROR" else self.TEXT)
        if model.status == "GAME_OVER":
            self.text(f"{model.score_p1}  :  {model.score_p2}", (400, 395), self.large, center=True)
        if model.status in ("ERROR", "GAME_OVER"):
            hint = "R: volver al registro y jugar otra vez   ·   Esc: salir"
        else:
            hint = "Abre otro cliente para empezar una partida.   ·   Esc: salir"
        self.text(hint, (400, 515), color=self.MUTED, center=True)

    def close(self):
        pygame.quit()

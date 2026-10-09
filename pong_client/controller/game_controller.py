import queue
import pygame
from model.game_state import GameStateModel
from view.renderer import GameView
from controller.input_handler import InputHandler
from net.network_service import NetworkService
from protocol import validate_profile

class GameController:
    def __init__(self, host, port, nickname="", email="", auto_connect=False):
        self.model = GameStateModel()
        self.view = GameView()
        self.input = InputHandler()
        self.net = None
        self.clock = pygame.time.Clock()
        self.fields = [host, str(port), nickname, email]
        self.focus = 2 if not nickname else 0
        self.form_error = ""
        self.auto_connect = auto_connect

    def _connect(self):
        host, port, nickname, email = self.fields
        try:
            if not host.strip():
                raise ValueError("Escribe la dirección del servidor.")
            port = int(port)
            if not 1 <= port <= 65535:
                raise ValueError("El puerto debe estar entre 1 y 65535.")
            validate_profile(nickname, email)
        except ValueError as error:
            self.form_error = str(error)
            self.model.status = "REGISTER"
            return
        if self.net:
            self.net.close()
        self.model = GameStateModel()
        self.model.status = "CONNECTING"
        self.model.status_message = "Conectando al servidor..."
        self.net = NetworkService(host.strip(), port)
        self.net.connect(nickname, email)
        self.form_error = ""
        pygame.key.stop_text_input()

    def _form_event(self, event):
        if event.type == pygame.MOUSEBUTTONDOWN and event.button == 1:
            for index, rect in enumerate(self.view.field_rects):
                if rect.collidepoint(event.pos):
                    self.focus = index
            if self.view.play_button.collidepoint(event.pos):
                self._connect()
        elif event.type == pygame.KEYDOWN:
            if event.key == pygame.K_TAB:
                self.focus = (self.focus + (-1 if event.mod & pygame.KMOD_SHIFT else 1)) % 4
            elif event.key == pygame.K_BACKSPACE:
                self.fields[self.focus] = self.fields[self.focus][:-1]
            elif event.key in (pygame.K_RETURN, pygame.K_KP_ENTER):
                self._connect()
        elif event.type == pygame.TEXTINPUT:
            limit = [253, 5, 24, 254][self.focus]
            candidate = self.fields[self.focus] + event.text
            if len(candidate.encode("utf-8")) <= limit:
                self.fields[self.focus] = candidate

    def run(self):
        running, last_direction = True, None
        pygame.key.start_text_input()
        if self.auto_connect:
            self._connect()
        try:
            while running:
                for event in pygame.event.get():
                    if event.type == pygame.QUIT or (event.type == pygame.KEYDOWN and event.key == pygame.K_ESCAPE):
                        running = False
                    elif self.model.status == "REGISTER":
                        self._form_event(event)
                    elif event.type == pygame.KEYDOWN and event.key == pygame.K_r and self.model.status in ("GAME_OVER", "ERROR"):
                        if self.net:
                            self.net.close()
                        self.net = None
                        self.model = GameStateModel()
                        self.form_error = ""
                        last_direction = None
                        pygame.key.start_text_input()
                if self.net:
                    # Bound work per rendered frame, so queued data cannot starve UI events.
                    for _ in range(128):
                        try:
                            kind, value = self.net.events.get_nowait()
                        except queue.Empty:
                            break
                        try:
                            if kind == "packet":
                                self.model.apply(*value)
                            elif kind == "error":
                                self.model.fail(value)
                            elif kind == "closed":
                                self.model.fail("Se perdió la conexión con el servidor.")
                        except ValueError as error:
                            self.model.fail(str(error))
                            self.net.close()
                    if self.model.status == "PLAYING":
                        direction = self.input.direction()
                        if direction != last_direction:
                            error = self.net.send_move(direction)
                            if error:
                                self.model.fail(error)
                            last_direction = direction
                self.view.render(self.model, self.fields, self.focus, self.form_error)
                self.clock.tick(60)
        finally:
            if self.net:
                self.net.close()
            self.view.close()

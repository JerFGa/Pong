import pygame
from model.game_state import GameStateModel
from view.renderer import GameView
from controller.input_handler import InputHandler
from net.network_service import NetworkService

class GameController:
    def __init__(self, host: str, port: int, nickname: str, email: str):
        self.model = GameStateModel()
        self.view = GameView()
        self.input_handler = InputHandler()
        self.net = NetworkService(host, port, self.model)
        self.clock = pygame.time.Clock()
        self.nickname = nickname
        self.email = email

    def run(self):
        # 1. Conectar y registrar
        connected = self.net.connect(self.nickname, self.email)
        if not connected:
            # Mostrar error unos segundos antes de salir
            for _ in range(120):
                self.view.render(self.model)
                self.clock.tick(60)
            self.view.close()
            return

        last_direction = -1
        running = True

        # Bucle principal de juego
        while running:
            # 2. Capturar entrada
            direction, quit_game = self.input_handler.get_input()
            if quit_game:
                running = False
                break

            # 3. Si el estado es activo y cambió de tecla, enviar al servidor
            if self.model.status == "PLAYING":
                if direction != last_direction:
                    self.net.send_move(direction)
                    last_direction = direction

            # 4. Renderizar Vista con el estado actual del Modelo
            self.view.render(self.model)
            self.clock.tick(60)

        # 5. Limpieza
        self.net.close()
        self.view.close()

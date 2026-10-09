import os
os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
import pathlib
import sys
import unittest
ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "pong_client"))
import pygame
from controller.game_controller import GameController

class UiTests(unittest.TestCase):
    def test_registration_keyboard_validation_and_render_states(self):
        app = GameController("127.0.0.1", 8080)
        self.addCleanup(app.view.close)
        app._form_event(pygame.event.Event(pygame.TEXTINPUT, text="Ana"))
        self.assertEqual(app.fields[2], "Ana")
        app._form_event(pygame.event.Event(pygame.KEYDOWN, key=pygame.K_TAB, mod=0))
        app._form_event(pygame.event.Event(pygame.TEXTINPUT, text="bad"))
        app._connect()
        self.assertEqual(app.model.status, "REGISTER")
        self.assertIn("correo", app.form_error)
        output = ROOT / "test-results" / "ui"
        output.mkdir(parents=True, exist_ok=True)
        app.fields[3] = "ana@eafit.edu.co"
        for state in ["REGISTER", "CONNECTING", "WAITING", "PLAYING", "GAME_OVER", "ERROR"]:
            app.model.status = state
            app.model.role = 1
            app.model.score_p1, app.model.score_p2 = 5, 3
            app.model.status_message = "¡Victoria!" if state == "GAME_OVER" else "Esperando a otro jugador..."
            if state == "ERROR":
                app.model.status_message = "No se pudo conectar con el servidor. Revisa la dirección y el puerto."
            app.view.render(app.model, app.fields, app.focus, "")
            pygame.image.save(app.view.screen, str(output / f"{state.lower()}.png"))

    def test_escape_exits_application(self):
        app = GameController("127.0.0.1", 8080)
        pygame.event.post(pygame.event.Event(pygame.KEYDOWN, key=pygame.K_ESCAPE))
        app.run()
        self.assertFalse(pygame.get_init())

if __name__ == "__main__":
    unittest.main()

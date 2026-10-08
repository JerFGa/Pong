import sys
from controller.game_controller import GameController

def main():
    host = sys.argv[1] if len(sys.argv) > 1 else "127.0.0.1"
    port = int(sys.argv[2]) if len(sys.argv) > 2 else 8080
    nickname = sys.argv[3] if len(sys.argv) > 3 else "Jugador1"
    email = sys.argv[4] if len(sys.argv) > 4 else "jugador1@eafit.edu.co"

    print(f"=== Pong Client (MVC) ===")
    print(f"Servidor: {host}:{port}")
    print(f"Perfil: Nick={nickname}, Email={email}")
    print("Iniciando aplicación...")

    app = GameController(host, port, nickname, email)
    app.run()

if __name__ == "__main__":
    main()

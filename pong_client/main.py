"""Run with no arguments for the registration form; four arguments connect directly."""
import argparse

def port_number(value):
    try:
        port = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("el puerto debe ser un entero") from error
    if not 1 <= port <= 65535:
        raise argparse.ArgumentTypeError("el puerto debe estar entre 1 y 65535")
    return port

def main():
    parser = argparse.ArgumentParser(description="Pong Online - cliente gráfico")
    parser.add_argument("host", nargs="?", default="127.0.0.1")
    parser.add_argument("port", nargs="?", default=8080, type=port_number)
    parser.add_argument("nickname", nargs="?", default="")
    parser.add_argument("email", nargs="?", default="")
    args = parser.parse_args()
    try:
        from controller.game_controller import GameController
    except ModuleNotFoundError as error:
        if error.name == "pygame":
            parser.exit(1, "Falta Pygame. Ejecuta: python -m pip install -r pong_client/requirements.txt\n")
        raise
    GameController(args.host, args.port, args.nickname, args.email,
                   auto_connect=bool(args.nickname and args.email)).run()

if __name__ == "__main__":
    main()

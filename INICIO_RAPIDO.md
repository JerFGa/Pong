# Iniciar Pong

Extrae el ZIP completo. No muevas archivos fuera de la carpeta del proyecto.
El servidor es Rust para Linux; el cliente es Python y puede ejecutarse en Windows.

## Windows con Ubuntu/WSL

Requisitos: Python 3.10 o posterior, Ubuntu en WSL y Rust/Cargo instalado dentro
de Ubuntu. En Ubuntu instala `build-essential` y Rust con las instrucciones del
README. Luego abre PowerShell en la carpeta que contiene este archivo.

Terminal 1, servidor:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start-server.ps1
```

Terminal 2, primer jugador:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start-client.ps1
```

Terminal 3, segundo jugador:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start-client.ps1
```

El script del cliente prepara su entorno e instala Pygame. Usa dos apodos distintos.
En cada ventana completa servidor, puerto, apodo y correo y pulsa Entrar a jugar.

## Linux

Después de instalar Rust, Make y Python:

```bash
source "$HOME/.cargo/env"
make
./server 8080 server.log
```

En otra terminal prepara el cliente:

```bash
python3 -m venv .venv
.venv/bin/python -m pip install -r pong_client/requirements.txt
.venv/bin/python pong_client/main.py
```

Abre un segundo cliente con el último comando desde otra terminal.
También se admite conexión directa:

```bash
.venv/bin/python pong_client/main.py 127.0.0.1 8080 Ana ana@eafit.edu.co
.venv/bin/python pong_client/main.py 127.0.0.1 8080 Luis luis@eafit.edu.co
```

Para equipos diferentes usa la IPv4 del servidor. La dirección `127.0.0.1` solo
sirve para la misma máquina. En AWS usa la IP pública y abre el puerto TCP del juego.

## Controles y reglas

- Flechas arriba/abajo o W/S: mover la paleta mientras mantienes la tecla.
- Soltar la tecla o quitar el foco de la ventana: detener la paleta.
- Gana el primero en llegar a cinco puntos.
- Si un jugador sale, el rival conectado recibe la victoria.
- R al terminar o ante un error: volver al registro y jugar otra partida.
- Esc: cerrar el cliente. Ctrl+C en la terminal del servidor: detenerlo.

## Comprobar el proyecto

```bash
source "$HOME/.cargo/env"
make check
make test
make test-ui PYTHON=.venv/bin/python
```

Consulta `docs/testing.md` para los resultados de esta entrega y
`docs/aws-deployment.md` para desplegar en AWS Academy.

## Incorporar los archivos al repositorio

Conserva la carpeta `.git` de tu copia existente. Copia el contenido de este
proyecto sobre la raíz del repositorio, incluyendo `.github` y `.gitattributes`.
No copies `target`, entornos virtuales ni binarios generados. Revisa el diff,
ejecuta las comprobaciones y crea el commit en la rama de entrega.

El ZIP contiene el código completo y no contiene historial de Git ni credenciales.

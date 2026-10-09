# Verificación del proyecto

Fecha de ejecución local: **8 de octubre de 2026**.

## Entorno observado

- Servidor: Ubuntu 22.04.5 en WSL2, x86_64, Rust/Cargo 1.99.0.
- Cliente y renderizado: Windows, Python 3.13, Pygame 2.6.1.
- Protocolo e integración: Python 3 de Ubuntu, sockets TCP reales en loopback.
- Compilación release con `Cargo.lock`; transporte `libc` 0.2.190.

## Resultados

| Verificación | Resultado |
|---|---|
| `cargo fmt -- --check` | Correcto |
| Clippy, todos los targets, `-D warnings` | Sin advertencias |
| Compilación release mediante `make` | Correcta |
| Pruebas unitarias Rust | 7/7 |
| Pruebas del cliente y protocolo Python | 6/6 |
| Integración TCP con el ejecutable Rust | 14/14 |
| Interfaz Pygame con SDL sin ventana | 2/2 |

**Total: 29 pruebas automatizadas satisfactorias.**
Las seis pruebas Python también se ejecutaron en Windows.

## Qué se probó

- Registro UTF-8 con longitudes por bytes y validación de apodo/correo.
- Cabeceras, tamaños, direcciones y estados fuera de rango.
- Representación big-endian de los 10 bytes de estado.
- Tramas fragmentadas byte a byte y tramas concatenadas en una escritura.
- Movimiento continuo tras un único comando y detención tras dirección cero.
- Límites de cancha, rebote contra pared y colisión de paleta a alta velocidad.
- Una partida completa hasta cinco puntos, con ambos clientes recibiendo el mismo
  ganador y marcador final antes del cierre.
- Dos salas simultáneas: mover/desconectar en una no altera la otra.
- Salida de un jugador en espera y emparejamiento posterior.
- Apodo duplicado sin expulsar al jugador que ya esperaba.
- Movimiento inválido y desconexión: victoria del rival conectado.
- Rechazo de movimientos antes del inicio y del exceso de 240 mensajes/s.
- Registro parcial que agota su plazo sin detener otras partidas.
- Capacidad máxima de 255 conexiones y recuperación al liberarlas.
- Reutilización de identificadores después de 270 sesiones sucesivas.
- Cliente `NetworkService` real de Python hablando con el servidor Rust.
- Logs con registros, respuestas, entradas, estados y goles.
- Ctrl+C/SIGTERM: los servidores de prueba terminan sin pánico ni bloqueo.
- Formulario, edición de campos, validación y cierre con Escape.
- Renderizado de registro, conexión, espera, cancha, resultado y error.

Se inspeccionaron visualmente las imágenes de registro, cancha y error generadas
por `test_ui.py`, sin recortes ni superposiciones observados. Las imágenes quedan
en `test-results/ui/` y no se versionan.

## Repetir las comprobaciones

Desde Ubuntu/WSL, en la raíz:

```bash
source "$HOME/.cargo/env"
make check
make test
make test-ui PYTHON=.venv/bin/python
```

El último comando requiere un entorno Python Linux con Pygame. No reutilices
una `.venv` creada en Windows desde Linux. Para probar la interfaz desde Windows:

```powershell
.\.venv\Scripts\python.exe -m unittest discover -s tests -p test_ui.py -v
```

Cada prueba de integración inicia un servidor propio en un puerto local disponible
y una bitácora temporal. No necesita un servidor externo ni AWS. La prueba de
capacidad reserva conexiones; no simula 127 partidas a carga sostenida.

## Pendientes de validación externa

- Crear y desplegar una instancia en AWS Academy siguiendo `aws-deployment.md`.
- Demostrar el juego entre equipos distintos a través de Internet.
- Medir latencia y rendimiento en la instancia real si se requieren cifras.

Estos puntos no se presentan como verificados. Tampoco se afirma compatibilidad
nativa del servidor con Windows o macOS: la plataforma implementada es Linux.

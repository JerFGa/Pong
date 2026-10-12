# Verificación del proyecto

Fecha de ejecución de esta entrega: **12 de octubre de 2026**.

## Entorno observado

- Servidor: Linux 6.18.44, x86_64, Rust/Cargo 1.99.0.
- Cliente y renderizado: Linux, Python 3.12.14, Pygame 2.6.1 y SDL sin ventana.
- Protocolo e integración: sockets TCP reales en loopback contra el servidor compilado.
- Compilación release con `Cargo.lock`; transporte `libc` 0.2.190.

## Resultados

| Verificación | Resultado |
|---|---|
| `cargo fmt -- --check` | Correcto |
| Clippy, todos los targets, `-D warnings` | Sin advertencias |
| Compilación release mediante `make` | Correcta |
| Pruebas unitarias Rust | 7/7 |
| Pruebas del cliente y protocolo Python | 6/6 |
| Integración TCP con el ejecutable Rust | 16/16 |
| Interfaz Pygame con SDL sin ventana | 2/2 |

**Total: 31 pruebas automatizadas satisfactorias; cero fallos.**
La integración completa tardó 36,799 segundos. Se ejecutaron los comandos
`make check`, `make test` y `make test-ui` sobre el código incluido en este ZIP.
Se usó `CARGO=/root/.cargo/bin/cargo` en este entorno para localizar Cargo.
No se ha repetido esta verificación en Windows; los scripts PowerShell se conservan.

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
- Desconexión del segundo jugador, victoria del primero y nuevo registro con
  los mismos apodos después de liberar la sala.
- Movimiento incompleto que agota su plazo sin afectar otra partida activa.
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
por `test_ui.py`, sin recortes ni superposiciones observados. Las seis imágenes
de esta verificación están incluidas en `evidencias/ui/`. Al repetir las pruebas,
las imágenes nuevas se generan en `test-results/ui/`.

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

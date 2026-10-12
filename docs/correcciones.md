# Correcciones de la entrega

Base: `JerFGa/Pong`, rama `Development`, commit
`f211b6b1710501f2d429a8cf8d66a3d2c2446eab`, revisado el 12 de octubre de 2026.
La entrega integra esa implementación completa; no usa la versión anterior de `main`.

## 3.2.1 — Cliente, servidor, registro y juego

- Dirección persistente en el servidor: una tecla mantenida mueve la paleta
  en cada tick; soltarla o perder foco envía dirección cero.
- El servidor fija la velocidad. Enviar más comandos no permite acelerar la paleta.
- Validación de nickname y correo, UTF-8 y longitudes por bytes.
- Apodos únicos durante sesiones activas y recuperación al salir.
- Cola con referencias débiles: un jugador desconectado no queda retenido en espera.
- Victoria del rival ante desconexión y visualización del resultado en el cliente.
- Cinco puntos para ganar; se transmite el marcador final antes de GAME_OVER.
- Se conservan formulario, conexión directa, flechas/W/S, puntuación, revancha con R
  y cierre con Escape.

## 3.2.2 — Protocolo

- Lecturas completas de cabecera y carga, tanto con TCP fragmentado como concatenado.
- Validación de opcode, tamaño, dirección, rol, ganador y fase del juego.
- Límites de carga útil y plazos para registro y mensajes incompletos.
- Formato binario compartido Rust/Python, con cabecera de tres bytes y big-endian.
- Especificación completa, ejemplos y diagramas conservados en README y arquitectura.

## 3.2.3 — Concurrencia

- Física independiente por sala y lector independiente por sesión.
- Sockets compartidos con Arc/OwnedFd: cierre único y shutdown para despertar lectores.
- Recolección de hilos terminados y apagado ordenado con Ctrl+C/SIGTERM.
- IDs reservados entre 1 y 255, liberados al salir; sin desbordamiento a cero.
- Límite explícito de capacidad y de frecuencia de entradas.

## 4 — Red, logger y ejecución

- Llamadas Berkeley directas mediante libc, sin frameworks de transporte.
- Envíos completos, plazos y protección frente a SIGPIPE.
- Logger serializado hacia consola y archivo, con registro de movimientos,
  estados, puntos, respuestas y resultados.
- Mejora adicional de esta entrega: las respuestas de registro se marcan como
  enviadas solo después de completar la escritura. Los fallos de REGISTER_RESP,
  GAME_START, GAME_STATE y GAME_OVER se registran como errores.
- Makefile, archivo Cargo.lock y comando `./server <PORT> <Log File>` conservados.
- Guía y configuración systemd/logrotate para AWS incluidas.

## 5 — Documentación y entrega

- Introducción, desarrollo, conclusiones y referencias completos.
- Guía de inicio rápido para Linux y Windows/WSL.
- GitHub Actions admite main, Development y ejecución manual; guarda evidencia UI.
- Dos pruebas adicionales: salida del segundo jugador con registro posterior y
  expiración de un movimiento incompleto manteniendo otra sala activa.
- Resultado local: 31 pruebas satisfactorias y Clippy sin advertencias.

## Acciones externas que los archivos no pueden realizar

- Desplegar y demostrar el juego en una instancia real de AWS Academy.
- Cambiar el repositorio a privado y habilitar el acceso del profesor.
- Subir esta versión a la rama de entrega e integrarla en main si corresponde.
- Completar los nombres reales de los integrantes en `docs/equipo.md`.
- Confirmar la interpretación de las notas contradictorias sobre C/Rust y
  clases de sockets. Esta entrega conserva Rust y llamadas Berkeley directas.

No se afirma que estas acciones externas estén realizadas.

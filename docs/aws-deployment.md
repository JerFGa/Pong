# Despliegue en AWS Academy

Estado: **guía preparada; despliegue pendiente de crear una instancia**.
No se han creado recursos ni validado conectividad pública en esta entrega local.

## 1. Preparar el laboratorio

1. En AWS Academy, inicia el Learner Lab y abre la consola AWS de esa sesión.
2. En EC2, crea una instancia **Ubuntu Server 24.04 LTS**, arquitectura x86_64,
   con un tipo pequeño permitido por tu laboratorio. Las opciones y cuotas
   dependen de la cuenta; usa las autorizadas por el curso.
3. Habilita IPv4 pública y guarda el par de claves de acceso. No subas el archivo
   `.pem`, contraseñas ni credenciales AWS al repositorio.
4. Configura el grupo de seguridad de la instancia:

| Uso | Protocolo/puerto | Origen |
|---|---|---|
| Administración | TCP 22 | Tu IPv4 pública con `/32` |
| Pong | TCP 8080 | IPv4 públicas de los jugadores con `/32` |

Para una demostración con direcciones desconocidas puedes permitir temporalmente
TCP 8080 desde `0.0.0.0/0`; vuelve a restringirlo al terminar. No hace falta UDP.
Si activas un firewall dentro de Ubuntu, también debe permitir 22 y 8080/TCP.
Las reglas funcionan según la documentación de
[grupos de seguridad de AWS](https://docs.aws.amazon.com/vpc/latest/userguide/security-group-rules.html).

## 2. Acceder y preparar Ubuntu

En tu PC, sustituye clave e IP por los valores reales:

```powershell
ssh -i C:\ruta\clave.pem ubuntu@IP_PUBLICA
```

En la instancia:

```bash
sudo apt update
sudo apt install -y build-essential curl git python3
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh
sh /tmp/rustup-init.sh -y --profile minimal --component rustfmt --component clippy
source "$HOME/.cargo/env"
```

## 3. Obtener exclusivamente Development

Si los cambios ya están subidos a GitHub:

```bash
git clone --branch Development --single-branch https://github.com/JerFGa/Pong.git
cd Pong
git branch --show-current
make check
make test
```

Para un repositorio privado necesitarás autenticación autorizada, preferiblemente
SSH con una clave de despliegue de solo lectura. No incluyas tokens en comandos
que queden guardados en el historial. Si los cambios solo están en tu PC, súbelos
a `Development` antes de clonar o transfiere una copia del proyecto sin `.git`,
`.venv`, `target`, logs ni credenciales mediante SCP.

Prueba inicial en primer plano:

```bash
./server 8080 server.log
```

Desde dos clientes en tu PC, usa la IP pública y apodos distintos. Comprueba que
los dos entren a la misma partida, vean el mismo marcador y reciban el resultado.
Ctrl+C detiene esta instancia de prueba antes de instalar el servicio.

## 4. Instalar como servicio

Desde la raíz del proyecto en Ubuntu, después de compilar con `make`:

```bash
getent passwd pong || sudo useradd --system --home /opt/pong --shell /usr/sbin/nologin pong
sudo install -d -o root -g root -m 755 /opt/pong
sudo install -d -o pong -g pong -m 750 /var/log/pong
sudo install -m 755 server /opt/pong/server
sudo install -m 644 deploy/pong.service /etc/systemd/system/pong.service
sudo install -m 644 deploy/pong.logrotate /etc/logrotate.d/pong
sudo systemctl daemon-reload
sudo systemctl enable --now pong
sudo systemctl status pong --no-pager
```

El servicio ejecuta `/opt/pong/server 8080 /var/log/pong/server.log` como usuario
sin privilegios. Reinicia ante fallos y se detiene con SIGTERM. Para cambiar el
puerto, modifica `ExecStart` y la regla correspondiente del grupo de seguridad.

```bash
sudo journalctl -u pong -f
sudo tail -f /var/log/pong/server.log
```

La rotación incluida conserva siete archivos comprimidos y revisa tamaño/fecha
cuando se ejecuta logrotate. `maxsize 20M` no es un límite instantáneo: depende
de la frecuencia del servicio de rotación. `copytruncate` conserva el descriptor
abierto; puede perder un pequeño intervalo de registros al rotar.

Para actualizar el ejecutable, compila y prueba primero; luego:

```bash
sudo systemctl stop pong
sudo install -m 755 server /opt/pong/server
sudo systemctl start pong
```

## 5. Verificación y evidencia de entrega

- `git branch --show-current` muestra `Development`.
- `make check` y `make test` terminan sin errores.
- `systemctl status pong` indica servicio activo.
- `ss -ltn` muestra el puerto 8080 en escucha.
- Dos equipos externos se registran y completan una partida.
- Cuatro clientes crean dos partidas independientes.
- Cerrar un cliente durante una partida notifica victoria al rival.
- La bitácora contiene registros, entradas, estados, goles y GAME_OVER.

Guarda capturas y un fragmento de log usando perfiles ficticios. Anota la fecha,
IP utilizada, versión del código y resultado observado; no afirmes pruebas que
no hayas ejecutado. Añade los resultados reales a `docs/testing.md`.

## 6. Problemas frecuentes y cierre

**Timeout al conectar:** verifica IP pública actual, estado del laboratorio,
grupo de seguridad, puerto del servicio y firewall local.

**Connection refused:** el servidor no escucha o se detuvo; consulta journalctl.

**Address already in use:** queda otra instancia manual escuchando en 8080;
detén esa instancia antes de iniciar systemd.

**Cargo no encontrado:** ejecuta `source "$HOME/.cargo/env"`.

Al terminar la demostración, detén el servicio con `sudo systemctl stop pong` y
detén la instancia/laboratorio según las instrucciones del curso. Los recursos
pueden consumir el presupuesto del laboratorio mientras siguen activos. La IP
pública puede cambiar tras detener/iniciar la instancia.

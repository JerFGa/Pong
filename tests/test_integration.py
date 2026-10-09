"""Real TCP tests against the compiled Rust server. No third-party packages."""
import os
import pathlib
import select
import socket
import struct
import subprocess
import tempfile
import time
import unittest
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "pong_client"))
from net.network_service import NetworkService
from model.game_state import GameStateModel
SERVER = pathlib.Path(os.environ.get("PONG_SERVER", ROOT / "server")).resolve()

def frame(opcode, payload=b""):
    return struct.pack("!BH", opcode, len(payload)) + payload

def exact(sock, length):
    data = bytearray()
    while len(data) < length:
        part = sock.recv(length - len(data))
        if not part:
            raise EOFError("server closed connection")
        data.extend(part)
    return bytes(data)

def receive(sock):
    opcode, length = struct.unpack("!BH", exact(sock, 3))
    return opcode, exact(sock, length)

def until(sock, wanted, timeout=4):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        sock.settimeout(max(0.1, end - time.monotonic()))
        opcode, payload = receive(sock)
        if opcode == wanted:
            return payload
    raise AssertionError(f"missing opcode {wanted}")

class ServerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.log = pathlib.Path(self.temp.name) / "server.log"
        with socket.socket() as reserve:
            reserve.bind(("127.0.0.1", 0))
            self.port = reserve.getsockname()[1]
        self.clients = []
        self.proc = subprocess.Popen([str(SERVER), str(self.port), str(self.log)], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        for _ in range(100):
            if self.proc.poll() is not None:
                self.fail(self.proc.stderr.read().decode())
            if self.log.exists() and "listo" in self.log.read_text():
                break
            time.sleep(0.02)
        else:
            self.fail("server startup timed out")

    def tearDown(self):
        for client in self.clients:
            client.close()
        self.proc.terminate()
        try:
            _, stderr = self.proc.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            self.proc.communicate()
            self.fail("server did not shut down gracefully")
        self.assertEqual(self.proc.returncode, 0, stderr.decode())
        self.assertNotIn("panicked", stderr.decode())
        self.temp.cleanup()

    def connect(self):
        sock = socket.create_connection(("127.0.0.1", self.port), timeout=3)
        sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        self.clients.append(sock)
        return sock

    def register(self, name, fragmented=False):
        sock = self.connect()
        nick = name.encode()
        packet = frame(1, bytes([len(nick)]) + nick + b"\x07a@b.com")
        if fragmented:
            for byte in packet:
                sock.sendall(bytes([byte]))
                time.sleep(0.002)
        else:
            sock.sendall(packet)
        response = until(sock, 2)
        self.assertEqual(response[0], 0)
        self.assertGreater(response[1], 0)
        return sock

    def pair(self, first="Ana", second="Luis"):
        a = self.register(first)
        self.assertEqual(until(a, 3), b"")
        b = self.register(second)
        self.assertEqual(until(a, 4), b"\x01")
        self.assertEqual(until(b, 4), b"\x02")
        return a, b

    def test_fragmented_input_continues_and_stop_holds(self):
        a = self.register("José", fragmented=True)
        until(a, 3)
        b = self.register("Luis")
        until(a, 4); until(b, 4)
        initial = struct.unpack("!HHHHBB", until(a, 6))[0]
        for byte in frame(5, b"\x02"):
            a.sendall(bytes([byte]))
            time.sleep(0.003)
        values = [struct.unpack("!HHHHBB", until(a, 6))[0] for _ in range(12)]
        self.assertGreater(values[-1], initial + 35)
        a.sendall(frame(5, b"\x00"))
        stopped = [struct.unpack("!HHHHBB", until(a, 6))[0] for _ in range(8)]
        self.assertEqual(stopped[-1], stopped[-3])

    def test_two_rooms_are_independent(self):
        a, b = self.pair("A", "B")
        c, d = self.pair("C", "D")
        a.sendall(frame(5, b"\x02"))
        a_states = [until(a, 6) for _ in range(15)]
        c_states = [until(c, 6) for _ in range(15)]
        self.assertGreater(struct.unpack("!HHHHBB", a_states[-1])[0], 300)
        self.assertTrue(all(struct.unpack("!HHHHBB", p)[0:2] == (255,255) for p in c_states))
        a.close()
        self.assertEqual(until(b, 7), b"\x02")
        self.assertEqual(len(until(d, 6)), 10)

    def test_abandoned_waiter_does_not_poison_matchmaking(self):
        waiting = self.register("Gone")
        until(waiting, 3)
        waiting.close()
        time.sleep(0.1)
        self.pair()

    def test_bad_registration_and_oversized_headers_are_rejected(self):
        for packet in [frame(1, b"\0\0"), frame(1, b"\x01\xff\x07a@b.com"),
                       frame(1, b"\x03Ana\x07a@b.comX"), b"\x01\xff\xff", b"\x77\0\0"]:
            sock = self.connect()
            sock.sendall(packet)
            self.assertEqual(until(sock, 2), b"\x01\0")
            sock.close()
        self.pair()

    def test_duplicate_name_keeps_original_waiter(self):
        a = self.register("Ana")
        until(a, 3)
        duplicate = self.connect()
        duplicate.sendall(frame(1, b"\x03ANA\x07a@b.com"))
        self.assertEqual(until(duplicate, 2), b"\x02\0")
        b = self.register("Luis")
        self.assertEqual(until(a, 4), b"\x01")
        self.assertEqual(until(b, 4), b"\x02")

    def test_invalid_movement_awards_surviving_player(self):
        a, b = self.pair()
        a.sendall(frame(5, b"\x03"))
        self.assertEqual(until(b, 7), b"\x02")

    def test_input_outside_match_is_rejected(self):
        a = self.register("Waiting")
        until(a, 3)
        a.sendall(frame(5, b"\x01"))
        self.assertEqual(a.recv(1), b"")
        self.pair()

    def test_coalesced_frames_and_log(self):
        a, _ = self.pair()
        a.sendall(frame(5, b"\x01") + frame(5, b"\x02") + frame(5, b"\x00"))
        for _ in range(5):
            until(a, 6)
        text = self.log.read_text()
        for event in ["REGISTER_REQ", "REGISTER_RESP", "WAIT_MATCH", "GAME_START", "MOVE_INPUT", "GAME_STATE"]:
            self.assertIn(event, text)

    def test_ids_are_recycled_after_more_than_255_sessions(self):
        for index in range(270):
            sock = self.register(f"P{index}")
            until(sock, 3)
            sock.close()
            time.sleep(0.01)
        self.pair()

    def test_full_match_reaches_five_and_reports_final_score(self):
        a, b = self.pair()
        scores = {a: None, b: None}
        winners = {}
        deadline = time.monotonic() + 35
        while len(winners) < 2 and time.monotonic() < deadline:
            ready, _, _ = select.select([s for s in (a,b) if s not in winners], [], [], 1)
            for sock in ready:
                opcode, payload = receive(sock)
                if opcode == 6:
                    scores[sock] = struct.unpack("!HHHHBB", payload)[-2:]
                elif opcode == 7:
                    winners[sock] = payload[0]
        self.assertEqual(len(winners), 2)
        self.assertEqual(winners[a], winners[b])
        self.assertEqual(scores[a], scores[b])
        self.assertEqual(scores[a][winners[a] - 1], 5)
        self.assertIn("gol marcador=", self.log.read_text())

    def test_real_python_client_and_rust_server(self):
        services = [NetworkService("127.0.0.1", self.port) for _ in range(2)]
        models = [GameStateModel(), GameStateModel()]
        try:
            for index, service in enumerate(services):
                models[index].status = "CONNECTING"
                service.connect(f"Client{index}", "a@b.com")
            for service, model in zip(services, models):
                while model.status != "PLAYING":
                    kind, value = service.events.get(timeout=3)
                    self.assertEqual(kind, "packet", value)
                    model.apply(*value)
            survivor_role = models[1].role
            services[0].close()
            while models[1].status != "GAME_OVER":
                kind, value = services[1].events.get(timeout=3)
                self.assertEqual(kind, "packet", value)
                models[1].apply(*value)
            self.assertEqual(models[1].winner, survivor_role)
        finally:
            for service in services:
                service.close()

    def test_incomplete_registration_expires(self):
        slow = self.connect()
        slow.sendall(b"\x01\x00")
        self.pair()
        self.assertEqual(until(slow, 2, timeout=7), b"\x01\0")

    def test_excessive_input_is_rejected_without_affecting_server(self):
        a, b = self.pair()
        a.sendall(frame(5, b"\0") * 241)
        self.assertEqual(until(b, 7), b"\x02")
        self.pair("New1", "New2")

    def test_capacity_is_bounded_and_server_recovers(self):
        # Keep handshakes pending: all 255 reservations must be unique and bounded.
        pending = [self.connect() for _ in range(255)]
        overflow = self.connect()
        self.assertEqual(until(overflow, 2), b"\x03\0")
        for sock in pending:
            sock.close()
        time.sleep(0.2)
        self.pair()

if __name__ == "__main__":
    unittest.main()

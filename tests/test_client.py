import pathlib
import socket
import sys
import threading
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / "pong_client"))
from protocol import *
from model.game_state import GameStateModel
from net.network_service import NetworkService


class ClientProtocolTests(unittest.TestCase):
    def test_registration_uses_utf8_byte_lengths(self):
        self.assertEqual(register_packet("José", "a@b.com"), b"\x01\x00\x0e\x05Jos\xc3\xa9\x07a@b.com")

    def test_profile_validation(self):
        for nick, email in [("", "a@b.com"), ("a" * 25, "a@b.com"), ("\nX", "a@b.com"),
                            ("Ana", "bad"), ("Ana", "x@@e.co"), ("Ana", "x@y."), (" Ana", "a@b.com")]:
            with self.subTest(nick=nick, email=email), self.assertRaises(ValueError):
                register_packet(nick, email)

    def test_response_lengths_and_ranges(self):
        for header in [b"\x06\x00\x09", b"\xff\x00\x00", b"\x07\xff\xff"]:
            with self.assertRaises(ValueError):
                decode_header(header)
        self.assertEqual(decode_payload(GAME_STATE, bytes.fromhex("00ff00ff018a01260000")), (255, 255, 394, 294, 0, 0))
        for opcode, payload in [(GAME_START, b"\x03"), (REGISTER_RESP, b"\0\0"), (GAME_STATE, b"\xff" * 10)]:
            with self.assertRaises(ValueError):
                decode_payload(opcode, payload)

    def test_state_machine_and_terminal_result(self):
        model = GameStateModel()
        model.status = "CONNECTING"
        model.apply(REGISTER_RESP, (0, 1))
        model.apply(WAIT_MATCH, ())
        model.apply(GAME_START, (1,))
        model.apply(GAME_STATE, (255, 255, 394, 294, 5, 3))
        model.apply(GAME_OVER, (1,))
        model.fail("EOF after result")
        self.assertEqual(model.status, "GAME_OVER")
        self.assertEqual(model.score_p1, 5)
        with self.assertRaises(ValueError):
            model.apply(GAME_START, (2,))

    def test_rejected_registration_survives_eof(self):
        model = GameStateModel()
        model.status = "CONNECTING"
        model.apply(REGISTER_RESP, (2, 0))
        model.fail("EOF")
        self.assertIn("apodo", model.status_message)

    def test_network_receives_fragmented_and_coalesced_frames(self):
        listener = socket.socket()
        self.addCleanup(listener.close)
        listener.bind(("127.0.0.1", 0))
        listener.listen()
        errors = []
        def fake_server():
            try:
                conn, _ = listener.accept()
                with conn:
                    conn.recv(512)
                    for chunk in [b"\x02", b"\x00", b"\x02\x00\x01\x03\x00\x00", b"\x04\x00\x01\x01\x07\x00\x01\x01"]:
                        conn.sendall(chunk)
            except Exception as error:
                errors.append(error)
        thread = threading.Thread(target=fake_server, daemon=True)
        thread.start()
        service = NetworkService("127.0.0.1", listener.getsockname()[1])
        self.addCleanup(service.close)
        service.connect("Ana", "a@b.com")
        packets = []
        while True:
            kind, value = service.events.get(timeout=3)
            self.assertNotEqual(kind, "error", value)
            if kind == "closed":
                break
            packets.append(value[0])
        thread.join(timeout=1)
        self.assertFalse(errors)
        self.assertEqual(packets, [REGISTER_RESP, WAIT_MATCH, GAME_START, GAME_OVER])


if __name__ == "__main__":
    unittest.main()

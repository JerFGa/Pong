CARGO ?= cargo
PYTHON ?= python3
PORT ?= 8080
LOG ?= server.log
SERVER_DIR := pong_server
TARGET := server

.PHONY: all build run test test-unit test-integration test-ui check fmt clean help
all: build
build:
	$(CARGO) build --locked --release --manifest-path $(SERVER_DIR)/Cargo.toml
	cp $(SERVER_DIR)/target/release/pong_server ./$(TARGET)
run: build
	./$(TARGET) $(PORT) $(LOG)
test: test-unit test-integration
test-unit:
	$(CARGO) test --locked --manifest-path $(SERVER_DIR)/Cargo.toml
	$(PYTHON) -m unittest discover -s tests -p 'test_client.py' -v
test-integration: build
	PONG_SERVER=./server $(PYTHON) -m unittest discover -s tests -p 'test_integration.py' -v
test-ui:
	SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy $(PYTHON) -m unittest discover -s tests -p 'test_ui.py' -v
check:
	$(CARGO) fmt --manifest-path $(SERVER_DIR)/Cargo.toml -- --check
	$(CARGO) clippy --locked --manifest-path $(SERVER_DIR)/Cargo.toml --all-targets -- -D warnings
fmt:
	$(CARGO) fmt --manifest-path $(SERVER_DIR)/Cargo.toml
clean:
	$(CARGO) clean --manifest-path $(SERVER_DIR)/Cargo.toml
	rm -f ./$(TARGET)
help:
	@echo "make / make run PORT=8080 LOG=server.log"
	@echo "make test / make test-ui / make check / make fmt / make clean"

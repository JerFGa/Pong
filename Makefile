CARGO ?= cargo
SERVER_DIR := pong_server
TARGET := server

.PHONY: all build clean run help

all: build

build:
	@echo "Compilando PongServer..."
	$(CARGO) build --release --manifest-path $(SERVER_DIR)/Cargo.toml
	cp $(SERVER_DIR)/target/release/pong_server ./$(TARGET)
	@echo "Binario generado exitosamente en ./$(TARGET)"

clean:
	$(CARGO) clean --manifest-path $(SERVER_DIR)/Cargo.toml
	rm -f ./$(TARGET)

run: build
	./$(TARGET) 8080 server.log

help:
	@echo "Comandos disponibles:"
	@echo "  make        - Compila el servidor y genera ./server en la raíz"
	@echo "  make run    - Compila y ejecuta el servidor en el puerto 8080 con log server.log"
	@echo "  make clean  - Elimina compilados y el ejecutable ./server"

#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi
make
exec ./server "${1:-8080}" "${2:-server.log}"

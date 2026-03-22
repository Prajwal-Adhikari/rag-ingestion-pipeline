#!/bin/bash

set -e  # exit on any error

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SIDECAR_DIR="$PROJECT_ROOT/sidecar"
VENV="$SIDECAR_DIR/myenv"

echo "================================"
echo "  RAG Ingestion Pipeline"
echo "================================"

# Check .env exists
if [ ! -f "$PROJECT_ROOT/.env" ]; then
    echo "ERROR: .env file not found at project root"
    exit 1
fi
# Load .env
export $(grep -v '^#' "$PROJECT_ROOT/.env" | xargs)

# Check venv exists
if [ ! -d "$VENV" ]; then
    echo "ERROR: Python venv not found at $VENV"
    echo "Run: cd sidecar && python3 -m venv myenv && source myenv/bin/activate && pip install grpcio grpcio-tools wtpsplit"
    exit 1
fi

# Start Python sidecar in background
echo "[1/3] Starting gRPC sidecar..."
source "$VENV/bin/activate"
python3 "$SIDECAR_DIR/server.py" &
SIDECAR_PID=$!
echo "      Sidecar PID: $SIDECAR_PID"

# Wait for sidecar to be ready
echo "      Waiting for sidecar on port 50051..."
for i in {1..10}; do
    if nc -z localhost 50051 2>/dev/null; then
        echo "      Sidecar is ready."
        break
    fi
    sleep 1
    if [ $i -eq 10 ]; then
        echo "ERROR: Sidecar did not start in time"
        kill $SIDECAR_PID 2>/dev/null
        exit 1
    fi
done

# Start bot in background
echo "[2/3] Starting Telegram bot..."
cd "$PROJECT_ROOT"
RUST_LOG=info cargo run -p bot &
BOT_PID=$!
echo "      Bot PID: $BOT_PID"

# Start worker in background
echo "[3/3] Starting worker..."
RUST_LOG=info cargo run -p worker &
WORKER_PID=$!
echo "      Worker PID: $WORKER_PID"

echo ""
echo "All services running. Press Ctrl+C to stop."
echo ""

# On Ctrl+C kill everything
cleanup() {
    echo ""
    echo "Shutting down..."
    kill $SIDECAR_PID 2>/dev/null
    kill $BOT_PID 2>/dev/null
    kill $WORKER_PID 2>/dev/null
    echo "Done."
}
trap cleanup SIGINT SIGTERM

# Wait for all background processes
wait
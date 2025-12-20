#!/bin/bash
# Web-only dev (no Tauri): starts Rust web server + Vite dev server

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "╔════════════════════════════════════════╗"
echo "║   Council Of Dicks - Web Dev           ║"
echo "╚════════════════════════════════════════╝"
echo ""

echo "1️⃣  Starting backend (council-web-server) on :8080 ..."
pushd "$ROOT_DIR/src-tauri" >/dev/null
cargo run --bin council-web-server &
BACKEND_PID=$!
popd >/dev/null

echo "   ✅ Backend PID: $BACKEND_PID"

echo ""
echo "2️⃣  Starting Vite on 0.0.0.0:5175 ..."
echo "   URL: http://$(hostname -I | awk '{print $1}'):5175"
echo ""

default_cleanup() {
  echo ""
  echo "🛑 Stopping backend (PID $BACKEND_PID)"
  kill "$BACKEND_PID" 2>/dev/null || true
}
trap default_cleanup EXIT

cd "$ROOT_DIR"
pnpm dev --host 0.0.0.0 --port 5175

#!/bin/bash
# Start the standalone web server for browser-based access

SCRIPT_DIR="$(dirname "$0")"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$PROJECT_ROOT/logs"
LOG_FILE="$LOG_DIR/council-server.log"

# Ensure logs directory exists
mkdir -p "$LOG_DIR"

echo "🚀 Starting Council Of Dicks Web Server..."
echo "   Port: 8080"
echo "   Mode: Web (Headless)"
echo "   Logs: $LOG_FILE"

# Change to project root (where config/ is located)
cd "$PROJECT_ROOT"

# Check if release binary exists
if [ -f "src-tauri/target/release/council-web-server" ]; then
    echo "   Using: release binary"
    exec src-tauri/target/release/council-web-server 2>&1 | tee -a "$LOG_FILE"
else
    echo "   Using: cargo run (debug)"
    cd src-tauri
    exec cargo run --bin council-web-server 2>&1 | tee -a "$LOG_FILE"
fi

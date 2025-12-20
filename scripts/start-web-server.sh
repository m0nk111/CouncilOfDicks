#!/bin/bash
# Start the standalone web server for browser-based access

echo "🚀 Starting Council Of Dicks Web Server..."
echo "   Port: 8080"
echo "   Mode: Web (Headless)"

# Ensure we are in the right directory
cd "$(dirname "$0")/../src-tauri"

# Run the server
cargo run --bin council-web-server

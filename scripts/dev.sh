#!/bin/bash
# Development helper script

set -e

echo "╔════════════════════════════════════════╗"
echo "║   Council Of Dicks - Dev Startup       ║"
echo "╚════════════════════════════════════════╝"
echo ""

# Check Ollama connection
echo "1️⃣  Checking Ollama connection..."
if ./scripts/test-ollama.sh > /dev/null 2>&1; then
    echo "   ✅ Ollama is accessible"
else
    echo "   ⚠️  Warning: Ollama check failed"
    echo "   Continuing anyway..."
fi

echo ""
echo "2️⃣  Starting Tauri dev server..."
echo "   Port: 5174"
echo "   Hot reload: enabled"
echo ""

# Source cargo env
source "$HOME/.cargo/env" 2>/dev/null || true

# Run Tauri dev
pnpm tauri dev

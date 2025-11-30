#!/bin/bash
# Test Rust backend without GUI (for headless servers)

set -e

cd "$(dirname "$0")/.."

echo "╔════════════════════════════════════════╗"
echo "║   Backend Unit Tests (No GUI)         ║"
echo "╚════════════════════════════════════════╝"
echo ""

cd src-tauri

echo "🧪 Running Rust tests..."
cargo test --lib --color=always

echo ""
echo "✅ Backend tests passed!"
echo ""
echo "💡 Note: GUI tests require X11/Wayland display"
echo "   Run on desktop system to test full UI"

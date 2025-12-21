# 🛠️ Development Guide

## Setup Instructions

### 1. Install Prerequisites

#### Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Node.js & pnpm
```bash
# Install Node.js 20+ from nodejs.org or via nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20

# Install pnpm
npm install -g pnpm
```

#### Tauri Prerequisites

**Linux:**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

**macOS:**
```bash
xcode-select --install
```

**Windows:**
```powershell
# Install Microsoft C++ Build Tools
# https://visualstudio.microsoft.com/visual-cpp-build-tools/
```

## 📚 Related Documentation

- **[CONTRIBUTING.md](CONTRIBUTING.md)**: Guidelines for submitting your changes.
- **[HEADLESS.md](docs/HEADLESS.md)**: Development on headless servers.
- **[DOCKER.md](docs/DOCKER.md)**: Containerized development workflow.

### 2. Clone & Install

```bash
git clone https://github.com/YOUR_USERNAME/TheCouncelOfDicks.git
cd TheCouncelOfDicks
pnpm install
```

### 3. Configure Ollama Connection

The app connects to Ollama at `http://192.168.1.5:11434` by default.

**Test connection:**
```bash
pnpm test:ollama
# or
./scripts/test-ollama.sh
```

**Expected output:**
```
✅ NR5 IS ALIVE!
📋 Available models:
qwen2.5-coder:7b
...
```

### 4. Development Workflow

```bash
# Quick start with Ollama check
pnpm tauri:dev
# or
./scripts/dev.sh

# Standard dev mode
pnpm tauri dev

# Run Rust tests
cargo test --manifest-path=src-tauri/Cargo.toml

# Format code
cargo fmt --manifest-path=src-tauri/Cargo.toml

# Lint
cargo clippy --manifest-path=src-tauri/Cargo.toml
```

**Dev server:** http://localhost:5175  
**Hot reload:** Enabled for both Rust and frontend

### LAN access (browser on another device)

If you open the UI from another machine (e.g. `http://192.168.1.5:5175`) and get “Kan geen verbinding maken”, the dev server is either not running or it is only bound to localhost.

- Start dev: `pnpm tauri:dev` (or `pnpm dev` for web-only)
- Start dev: `pnpm tauri:dev` (Tauri) or `./scripts/dev-web.sh` (web-only; starts backend + Vite)
- Ensure Vite binds to all interfaces (already configured via `server.host: true` in `vite.config.ts`)
- If you use a firewall, allow TCP `5175`

If the UI loads but you see “Failed to load messages”, the Rust web server on `:8080` is not running. In web-mode the frontend calls `/api/...` and Vite proxies that to `http://127.0.0.1:8080`.

## Project Architecture

### Tauri IPC Commands

Communication between Rust and frontend happens via Tauri commands:

```rust
// src-tauri/src/commands.rs
#[tauri::command]
async fn start_council(question: String) -> Result<String, String> {
    // Implementation
}
```

```typescript
// src/lib/api.ts
import { invoke } from '@tauri-apps/api/tauri';

export async function startCouncil(question: string): Promise<string> {
  return await invoke('start_council', { question });
}
```

### State Management

Frontend uses Svelte stores for reactive state:

```typescript
// src/lib/stores/council.ts
import { writable } from 'svelte/store';

export const councilState = writable({
  active: false,
  question: '',
  rounds: [],
  verdict: null
});
```

### P2P Networking

libp2p handles all P2P communication:

```rust
// src-tauri/src/p2p/node.rs
pub struct TcodNode {
    peer_id: PeerId,
    swarm: Swarm<TcodBehaviour>,
}
```

## Testing

### Rust Unit Tests

```bash
cd src-tauri
cargo test
```

### Integration Tests

```bash
cargo test --test integration_tests
```

### Frontend Tests

```bash
pnpm test        # Run tests
pnpm test:watch  # Watch mode
```

## Building

### Development Build

```bash
pnpm tauri build --debug
```

### Production Build

```bash
pnpm tauri build
```

Outputs:
- **Linux**: `.deb`, `.AppImage` in `src-tauri/target/release/bundle/`
- **macOS**: `.app`, `.dmg` in `src-tauri/target/release/bundle/`
- **Windows**: `.msi`, `.exe` in `src-tauri/target/release/bundle/`

## Debugging

### Rust Backend

```bash
# Run with debug logs
RUST_LOG=debug pnpm tauri dev

# Use rust-lldb/rust-gdb
rust-lldb target/debug/council-of-dicks
```

### Frontend

Open DevTools in the Tauri window: `Ctrl+Shift+I` (Linux/Windows) or `Cmd+Opt+I` (macOS)

### Network Traffic

```bash
# Monitor P2P connections
RUST_LOG=libp2p=debug pnpm tauri dev
```

## Common Issues

### Build Fails on Linux

Make sure all webkit dependencies are installed:
```bash
sudo apt install libwebkit2gtk-4.1-dev
```

### Tauri Dev Window Won't Open

Clear cache and rebuild:
```bash
rm -rf node_modules .svelte-kit
pnpm install
pnpm tauri dev
```

### Rust Compilation Errors

Update Rust:
```bash
rustup update stable
```

## Performance Profiling

### Rust

```bash
cargo build --release
perf record -g ./target/release/council-of-dicks
perf report
```

### Frontend

Use Chrome DevTools Performance tab in the Tauri window.

## Documentation

Generate Rust docs:
```bash
cargo doc --open --manifest-path=src-tauri/Cargo.toml
```

## Release Process

1. Update version in:
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`

2. Create changelog entry

3. Build for all platforms

4. Create GitHub release

5. Upload binaries

## Getting Help

- **GitHub Discussions**: Ask questions
- **GitHub Issues**: Report bugs
- **Discord** (coming soon): Real-time chat

---

*Happy hacking!* 🚀

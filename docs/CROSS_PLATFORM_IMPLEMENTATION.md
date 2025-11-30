# 🖥️ The Council Of Dicks - Cross-Platform Implementation
*"One binary, all platforms, full node capabilities"*

## Solution: Tauri + Rust + Svelte

### Why Tauri?

```
┌────────────────────────────────────────────────────┐
│              TCOD Native App                        │
│                                                     │
│  ┌──────────────────────────────────────────────┐ │
│  │  Frontend (Svelte)                           │ │
│  │  - Chat UI                                   │ │
│  │  - Council visualization                     │ │
│  │  - Settings                                  │ │
│  └──────────────────┬───────────────────────────┘ │
│                     │ IPC (Tauri Commands)         │
│  ┌──────────────────▼───────────────────────────┐ │
│  │  Rust Backend                                │ │
│  │  ┌──────────────────────────────────────┐   │ │
│  │  │ P2P Network (libp2p)                 │   │ │
│  │  ├──────────────────────────────────────┤   │ │
│  │  │ Council Logic                        │   │ │
│  │  ├──────────────────────────────────────┤   │ │
│  │  │ Knowledge Bank (SQLite + IPFS)       │   │ │
│  │  ├──────────────────────────────────────┤   │ │
│  │  │ Ollama Client (192.168.1.5)          │   │ │
│  │  ├──────────────────────────────────────┤   │ │
│  │  │ Safety Systems (Dead Man's Switch)   │   │ │
│  │  └──────────────────────────────────────┘   │ │
│  └──────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────┘

        ↓ Compiles to ↓

Windows: tcod.exe (5-10MB)
macOS:   TCOD.app (5-10MB)  
Linux:   tcod (5-10MB)
```

**Advantages:**
- ✅ **Single codebase** → Windows, macOS, Linux
- ✅ **Native performance** → Rust backend
- ✅ **Small binary** → 5-10MB (vs Electron 100MB+)
- ✅ **Web technologies** → Modern UI with Svelte
- ✅ **System access** → Full filesystem, network, hardware
- ✅ **Auto-updates** → Built-in updater
- ✅ **Tray icon** → Runs in background

## Project Structure

```
tcod/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── p2p/            # P2P networking
│   │   │   ├── mod.rs
│   │   │   ├── node.rs     # libp2p node
│   │   │   ├── protocol.rs # TCOD protocol
│   │   │   └── discovery.rs
│   │   ├── council/        # Council logic
│   │   │   ├── mod.rs
│   │   │   ├── session.rs  # Active session
│   │   │   ├── voting.rs   # Consensus algorithms
│   │   │   └── personalities.rs
│   │   ├── knowledge/      # Knowledge bank
│   │   │   ├── mod.rs
│   │   │   ├── storage.rs  # SQLite + IPFS
│   │   │   └── graph.rs    # Knowledge graph
│   │   ├── safety/         # Safety systems
│   │   │   ├── mod.rs
│   │   │   ├── heartbeat.rs
│   │   │   └── challenges.rs
│   │   ├── ollama.rs       # Ollama API client
│   │   └── commands.rs     # Tauri commands (IPC)
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                    # Svelte frontend
│   ├── lib/
│   │   ├── components/
│   │   │   ├── Chat.svelte
│   │   │   ├── CouncilView.svelte
│   │   │   ├── PeerList.svelte
│   │   │   └── Settings.svelte
│   │   ├── stores/
│   │   │   ├── council.ts
│   │   │   ├── peers.ts
│   │   │   └── history.ts
│   │   └── api.ts          # Tauri command wrappers
│   ├── routes/
│   │   ├── +page.svelte    # Main chat
│   │   ├── history/
│   │   └── settings/
│   ├── app.html
│   └── app.css
│
├── package.json
├── svelte.config.js
├── vite.config.js
└── README.md
```

## Tech Stack Details

### Frontend
```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "svelte": "^4.0.0",
    "@sveltejs/kit": "^2.0.0",
    "marked": "^11.0.0",
    "highlight.js": "^11.0.0"
  }
}
```

### Backend (Rust)
```toml
[dependencies]
tauri = { version = "2.0", features = ["shell-open"] }
libp2p = { version = "0.54", features = ["tcp", "quic", "dns", "websocket"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
ipfs-api = "0.17"
ed25519-dalek = "2.0"  # For crypto signatures
rand = "0.8"
```

## Implementation Plan

### Phase 1: Basic Tauri App (Week 1)
```bash
# Install Tauri CLI
cargo install tauri-cli

# Create project
pnpm create tauri-app
# Choose:
# - Name: council-of-dicks
# - Frontend: SvelteKit
# - Package manager: pnpm
```

**Deliverable:**
- App window opens
- Basic Svelte UI
- Can call Rust functions from frontend

### Phase 2: Ollama Integration (Week 1-2)
```rust
// src-tauri/src/ollama.rs
pub struct OllamaClient {
    base_url: String,  // "http://192.168.1.5:11434"
    client: reqwest::Client,
}

#[tauri::command]
async fn query_model(
    model: String,
    prompt: String,
    system_prompt: String,
) -> Result<String, String> {
    // Call Ollama API
}
```

**Deliverable:**
- Connect to 192.168.1.5
- Query models from frontend
- Display responses

### Phase 3: P2P Network (Week 2-3)
```rust
// src-tauri/src/p2p/node.rs
pub struct TcodNode {
    peer_id: PeerId,
    swarm: Swarm<TcodBehaviour>,
    known_peers: HashMap<PeerId, PeerInfo>,
}

#[tauri::command]
async fn start_p2p_node() -> Result<String, String> {
    // Initialize libp2p
}

#[tauri::command]
async fn connect_to_peer(peer_addr: String) -> Result<(), String> {
    // Connect to another TCOD node
}
```

**Deliverable:**
- Two instances can connect P2P
- See peer list in UI
- Send messages between peers

### Phase 4: Council Logic (Week 3-4)
```rust
// src-tauri/src/council/session.rs
pub struct CouncilSession {
    id: Uuid,
    question: String,
    participants: Vec<AIModel>,
    rounds: Vec<DeliberationRound>,
    knowledge_context: Vec<Decision>,
}

#[tauri::command]
async fn start_council(
    question: String,
    human_signature: String,
) -> Result<SessionId, String> {
    // Verify human signature
    // Search knowledge bank for context
    // Start P2P council session
}

#[tauri::command]
async fn get_council_updates(
    session_id: String,
) -> Result<Vec<Message>, String> {
    // Stream deliberation to frontend
}
```

**Deliverable:**
- Multiple AI models debate
- See deliberation in real-time
- Reach consensus

### Phase 5: Knowledge Bank (Week 4-5)
```rust
// src-tauri/src/knowledge/storage.rs
pub struct KnowledgeBank {
    db: SqlitePool,
    ipfs: IpfsClient,
}

#[tauri::command]
async fn search_history(query: String) -> Result<Vec<Decision>, String> {
    // Semantic search through past decisions
}

#[tauri::command]
async fn save_decision(decision: Decision) -> Result<(), String> {
    // Save to local DB + IPFS
    // Broadcast to network
}
```

**Deliverable:**
- Decisions persist locally
- Can search history
- Syncs across network

### Phase 6: Safety Systems (Week 5-6)
```rust
// src-tauri/src/safety/heartbeat.rs
#[tauri::command]
async fn send_heartbeat(activity_proof: ActivityProof) -> Result<(), String> {
    // Update human presence timestamp
}

#[tauri::command]
async fn check_safety_status() -> Result<SafetyStatus, String> {
    // Return current safety state
}
```

**Deliverable:**
- Heartbeat monitoring works
- Dead man's switch triggers
- Can recover from shutdown

## Distribution Strategy

### Building for All Platforms

```bash
# Build for current platform
pnpm tauri build

# Cross-compile (from Linux)
pnpm tauri build --target x86_64-pc-windows-msvc
pnpm tauri build --target x86_64-apple-darwin
pnpm tauri build --target x86_64-unknown-linux-gnu
```

**Output:**
```
src-tauri/target/release/
├── bundle/
│   ├── deb/tcod_0.1.0_amd64.deb          # Linux
│   ├── appimage/tcod_0.1.0_amd64.AppImage # Linux
│   ├── msi/tcod_0.1.0_x64.msi            # Windows
│   ├── nsis/tcod_0.1.0_x64-setup.exe     # Windows
│   └── macos/TCOD.app                     # macOS
```

### Auto-Updates

```rust
// src-tauri/tauri.conf.json
{
  "updater": {
    "active": true,
    "endpoints": [
      "https://releases.tcod.network/{{target}}/{{current_version}}"
    ],
    "dialog": true,
    "pubkey": "YOUR_PUBLIC_KEY"
  }
}
```

### Distribution Channels

1. **GitHub Releases** (primary)
   - Automatic builds via GitHub Actions
   - Signed releases
   - Update manifest

2. **Direct Download** (website)
   - tcod.network/download
   - Auto-detect platform
   - Verify signatures

3. **Package Managers** (future)
   - Snap (Linux)
   - Homebrew (macOS)
   - Chocolatey (Windows)
   - Flatpak (Linux)

## Running as Background Service

```rust
// src-tauri/src/main.rs
fn main() {
    tauri::Builder::default()
        .system_tray(create_system_tray())
        .on_system_tray_event(handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            start_p2p_node,
            query_model,
            start_council,
            // ... all commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn create_system_tray() -> SystemTray {
    SystemTray::new().with_menu(
        SystemTrayMenu::new()
            .add_item(CustomMenuItem::new("show", "Show Window"))
            .add_item(CustomMenuItem::new("hide", "Hide Window"))
            .add_separator()
            .add_item(CustomMenuItem::new("peers", "3 Peers Connected"))
            .add_item(CustomMenuItem::new("status", "Council Active"))
            .add_separator()
            .add_item(CustomMenuItem::new("quit", "Quit"))
    )
}
```

**User Experience:**
- Install once
- Runs in system tray
- Always connected to P2P network
- Click to open UI when needed
- Contributes compute in background

## Web Version (Bonus)

For users who don't want to install:

```
┌─────────────────────────────────┐
│   tcod.network (web app)        │
│   - Light node (WebRTC only)    │
│   - Can't host models           │
│   - Can participate in councils │
│   - Browser-based P2P           │
└─────────────────────────────────┘
         ↕ WebRTC
┌─────────────────────────────────┐
│   Native app (full node)        │
│   - Hosts models                │
│   - Routes queries              │
│   - Full P2P capabilities       │
└─────────────────────────────────┘
```

## Development Workflow

```bash
# Install dependencies
pnpm install

# Run dev mode (hot reload for frontend + backend)
pnpm tauri dev

# Build for production
pnpm tauri build

# Run tests
cargo test --manifest-path=src-tauri/Cargo.toml
pnpm test
```

---

**Summary:**
- ✅ **Cross-platform**: One binary, runs everywhere
- ✅ **Native**: Rust performance + system access
- ✅ **Modern UI**: Svelte for beautiful interface
- ✅ **Small**: ~5-10MB binary
- ✅ **Background**: Runs as service/tray app
- ✅ **P2P**: Full node in every client
- ✅ **Easy distribution**: GitHub releases + auto-updates

Ready to `pnpm create tauri-app`? 🚀

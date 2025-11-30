# Council Of Dicks (TCOD)

![Version](https://img.shields.io/badge/version-0.3.0--alpha-orange)
![Status](https://img.shields.io/badge/status-alpha-yellow)
![License](https://img.shields.io/badge/license-Custom-blue)

> *"Democracy for AI - When one opinion isn't enough"*

A **decentralized P2P network** where multiple AI models deliberate until they reach consensus. Every client is also a server node (Tor-like architecture), creating a truly distributed AI democracy that serves humanity.

## 🚀 Current Status (v0.3.0-alpha)

✅ **Implemented:**
- Tauri 2.0 cross-platform application (Rust + Svelte 5)
- Ollama AI integration (local + network)
- P2P networking foundation (libp2p with gossipsub, mDNS, Kademlia DHT)
- Council deliberation system (multi-round voting + blind voting + consensus)
- Ed25519 cryptographic signatures (response authentication)
- MCP server integration (JSON-RPC 2.0 on port 9001)
- Comprehensive logging & metrics (debug mode + performance tracking)
- 36 backend tests passing

⏳ **In Development:**
- Council UI panel (frontend integration)
- Proof of Human Value (PoHV) safety mechanisms
- Reputation/ranking system (5-tier meritocracy)
- Knowledge bank persistence (SQLite + IPFS)

## 🌟 Core Philosophy

- **🔓 Free & Open**: No subscriptions, no paywalls, fully FOSS
- **🌐 Decentralized**: P2P network, no central authority
- **🔐 Human-Centric**: Multiple failsafes ensure AI cannot operate without human input
- **🧠 Eternal Memory**: Never-ending session where all decisions build on past knowledge
- **💪 Community-Owned**: Network grows stronger as more nodes join

## 🎯 What It Does

Instead of asking one AI and hoping for a good answer, TCOD:

1. **Submits your question** to multiple AI models simultaneously (via Ollama)
2. **Models deliberate** in rounds, challenging each other's reasoning
3. **Blind voting** with cryptographic commitments prevents gaming
4. **Reach consensus** through Byzantine fault-tolerant voting (67% threshold)
5. **Cryptographically signed responses** (Ed25519) - verify authenticity
6. **Transparent debate** - see full deliberation history with all arguments
7. **P2P distribution** - every node contributes to network resilience
8. **Build knowledge** - sessions preserved for future reference (eternal council)

### Current Capabilities (v0.3.0-alpha)

✅ **Ask Ollama models** - Query any Ollama-compatible AI model  
✅ **Create council sessions** - Multi-round deliberation with blind voting  
✅ **P2P networking** - Join mesh network, discover local peers  
✅ **Sign responses** - Cryptographic proof of response integrity  
✅ **MCP integration** - External AI agents can use council as a tool  
✅ **Performance metrics** - Track request times, rolling averages  
⏳ **Council UI** - Frontend interface for session management (in dev)  
⏳ **Knowledge bank** - Persistent decision history with semantic search (in dev)

## 🏗️ Architecture

```
┌──────────────────────────────────────────────────────────────┐
│              TCOD Cross-Platform App (v0.3.0-alpha)           │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Frontend (Svelte 5 + TypeScript)                      │ │
│  │  - Chat interface (in dev)                             │ │
│  │  - Real-time deliberation viewer                       │ │
│  │  - Network status & peer list                          │ │
│  │  - Council session management                          │ │
│  └────────────────────┬───────────────────────────────────┘ │
│                       │ Tauri IPC (19 commands)             │
│  ┌────────────────────▼───────────────────────────────────┐ │
│  │  Rust Backend (tokio async)                           │ │
│  │  ├─ P2P Network (libp2p 0.54)                         │ │
│  │  │  ├─ Gossipsub (pub/sub messaging)                  │ │
│  │  │  ├─ mDNS (local peer discovery)                    │ │
│  │  │  ├─ Kademlia DHT (distributed routing)             │ │
│  │  │  └─ Noise + Yamux (encryption + multiplexing)      │ │
│  │  ├─ Council Logic (multi-round deliberation)          │ │
│  │  │  ├─ Blind voting (cryptographic commitments)       │ │
│  │  │  ├─ Consensus calculation (67% threshold)          │ │
│  │  │  └─ Session management (create/vote/retrieve)      │ │
│  │  ├─ Cryptographic Signatures (Ed25519)                │ │
│  │  │  ├─ Response signing (50μs per signature)          │ │
│  │  │  ├─ Verification (150μs per check)                 │ │
│  │  │  └─ Identity management (keypair generation/load)  │ │
│  │  ├─ MCP Server (JSON-RPC 2.0 on port 9001)           │ │
│  │  │  └─ External tool integration for AI agents        │ │
│  │  ├─ Knowledge Bank (SQLite + IPFS - in dev)           │ │
│  │  ├─ Ollama Integration (qwen2.5-coder:7b)             │ │
│  │  ├─ Logger (emoji + color + timestamps)               │ │
│  │  ├─ Metrics (rolling average, 100 requests)           │ │
│  │  └─ Safety Systems (PoHV - in dev)                    │ │
│  └────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
                               │
                               │ Network Protocol (8 message types)
                               │
                      ┌────────▼─────────┐
                      │  P2P Mesh Network │
                      │  (Every client is │
                      │   also a node)    │
                      └──────────────────┘
```

## 🔐 Safety Features

### Implemented (v0.3.0-alpha)

✅ **Cryptographic Signatures (Ed25519)**
- AI responses are digitally signed
- Verify authenticity and integrity
- Prevent response spoofing and tampering
- 128-bit security level
- Public key fingerprints for identity verification

✅ **Blind Voting with Commitments**
- Prevents voting manipulation
- Cryptographic commitments before reveal
- Byzantine fault tolerance (67% consensus)
- Anti-gaming through reveal-commit protocol

✅ **Network Encryption**
- Noise protocol for P2P communication
- All messages encrypted in transit
- Peer authentication via libp2p

### Planned (Proof of Human Value - PoHV)

⏳ **Active Heartbeat**: Requires human interaction every 24 hours  
⏳ **Random Challenges**: CAPTCHA-like proofs at random intervals  
⏳ **Network Consensus**: Minimum percentage of human-operated nodes required  
⏳ **Resource Gates**: Humans control compute allocation  
⏳ **Dead Man's Switch**: Council pauses → read-only mode → requires human intervention

**Design Philosophy:** Multiple independent failsafes ensure AI cannot operate autonomously without humans. See [SAFETY_AND_MEMORY.md](docs/SAFETY_AND_MEMORY.md) for details.

## 🚀 Getting Started

### Prerequisites

- **Rust** (latest stable): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Node.js** (v20+): `https://nodejs.org/`
- **pnpm**: `npm install -g pnpm`
- **Ollama**: `https://ollama.ai/` (for AI model hosting)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/TheCouncelOfDicks.git
cd TheCouncelOfDicks

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Run tests
cargo test --manifest-path=src-tauri/Cargo.toml
pnpm test

# Build for production
pnpm tauri build
```

### Configuration

#### Ollama Connection
Configure your Ollama endpoint in `src-tauri/config.json`:

```json
{
  "ollama_url": "http://192.168.1.5:11434",
  "default_model": "qwen2.5-coder:7b",
  "debug_enabled": false
}
```

Or use Tauri commands to set at runtime:
```javascript
import { invoke } from '@tauri-apps/api/tauri';
await invoke('set_config', { key: 'ollama_url', value: 'http://localhost:11434' });
```

#### MCP Server
The MCP server starts automatically on port 9001 (localhost only). External AI agents can connect using JSON-RPC 2.0:

```bash
# Test MCP server
curl -X POST http://localhost:9001 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

#### P2P Networking
P2P network automatically discovers local peers via mDNS. Bootstrap nodes for wider discovery coming soon.

```javascript
// Start P2P node
await invoke('p2p_start', { listen_addr: '/ip4/0.0.0.0/tcp/0' });

// Get network status
const status = await invoke('p2p_status');
console.log(`Connected peers: ${status.connected_peers.length}`);
```

### Current Features (v0.3.0-alpha)

#### ✅ Ollama Integration
- Connect to local or network Ollama instances
- Support for any Ollama-compatible model
- Configurable timeouts and retries

#### ✅ P2P Networking
- libp2p-based mesh network
- mDNS peer discovery
- Kademlia DHT for routing
- Noise protocol encryption
- Gossipsub pub/sub messaging

#### ✅ Council Deliberation
- Create council sessions
- Multi-round deliberation
- Blind voting with cryptographic commitments
- 67% consensus threshold
- Session history and retrieval

#### ✅ Cryptographic Signatures
- Ed25519 digital signatures
- Response authentication (prevent spoofing)
- Identity management (keypair generation/loading)
- 128-bit security level
- Sub-millisecond performance (50μs sign, 150μs verify)

#### ✅ MCP Server
- JSON-RPC 2.0 interface
- External AI agent integration
- Tools: council_ask, council_get_session, council_list_sessions
- Localhost binding (security-first)

#### ✅ Logging & Metrics
- Emoji-prefixed debug output (🐛 🔍 ⚠️ ❌ ✅ 📊 🔧)
- Global debug toggle (runtime + persistent)
- Performance metrics with rolling averages
- Request/response timing

#### ⏳ In Development
- Council UI panel (Svelte frontend)
- Proof of Human Value (PoHV) safety mechanisms
- Reputation/ranking system (5-tier meritocracy)
- Knowledge bank persistence (SQLite + IPFS)

## 📁 Project Structure

```
TheCouncelOfDicks/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── p2p/            # P2P networking (libp2p)
│   │   ├── council/        # Council logic & voting
│   │   ├── knowledge/      # Knowledge bank & history
│   │   ├── safety/         # Safety systems
│   │   └── ollama.rs       # Ollama API client
│   └── Cargo.toml
│
├── src/                    # Svelte frontend
│   ├── lib/
│   │   ├── components/     # UI components
│   │   ├── stores/         # State management
│   │   └── api.ts          # Tauri command wrappers
│   └── routes/             # SvelteKit routes
│
├── docs/                   # Documentation
│   ├── COUNCIL_OF_DICKS_CONCEPT.md    # Original concept
│   ├── CORE_VISION.md                  # Core philosophy
│   ├── SAFETY_AND_MEMORY.md            # Safety systems
│   ├── ARCHITECTURE_DISCUSSION.md      # Tech stack evaluation
│   ├── CROSS_PLATFORM_IMPLEMENTATION.md # Tauri implementation
│   ├── AI_RANKING_SYSTEM.md            # 5-tier meritocracy
│   ├── ANTI_GAMING_MECHANISMS.md       # Defense layers
│   └── UI_UX_SPECS.md                  # UI design
│
├── README.md               # This file
├── CHANGELOG.md            # Version history
├── CONTRIBUTING.md         # Contribution guidelines
├── DEVELOPMENT.md          # Dev setup instructions
├── COMMERCIAL.md           # Commercial licensing options
└── LICENSE                 # Custom commercial license
```

## 🛠️ Development

See [DEVELOPMENT.md](docs/DEVELOPMENT.md) for detailed setup instructions.

### Quick Commands

```bash
# Development mode (hot reload)
pnpm tauri dev

# Enable debug logging
RUST_LOG=debug pnpm tauri dev

# Run backend tests (36 tests)
cargo test --manifest-path=src-tauri/Cargo.toml

# Run specific test module
cargo test --manifest-path=src-tauri/Cargo.toml crypto::tests

# Run frontend tests
pnpm test

# Format code
cargo fmt --manifest-path=src-tauri/Cargo.toml
cargo clippy --manifest-path=src-tauri/Cargo.toml
pnpm format
```

### Building

```bash
# Build for current platform
pnpm tauri build

# Output: src-tauri/target/release/bundle/
# - Windows: .msi installer
# - macOS: .app + .dmg
# - Linux: .deb, .AppImage
```

### Project Structure

```
TheCouncelOfDicks/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── lib.rs          # Main entry point (19 Tauri commands)
│   │   ├── state.rs        # Global application state
│   │   ├── config.rs       # Configuration management
│   │   ├── ollama.rs       # Ollama API client
│   │   ├── p2p.rs          # P2P networking (libp2p)
│   │   ├── protocol.rs     # P2P message protocol (8 types)
│   │   ├── council.rs      # Council deliberation + voting
│   │   ├── crypto.rs       # Ed25519 signatures (NEW v0.3.0)
│   │   ├── mcp.rs          # MCP JSON-RPC server (NEW v0.3.0)
│   │   ├── logger.rs       # Custom logging system
│   │   └── metrics.rs      # Performance metrics
│   ├── config.json         # Default configuration
│   └── Cargo.toml          # Rust dependencies
│
├── src/                    # Svelte 5 frontend
│   ├── lib/
│   │   ├── components/     # UI components
│   │   └── api.ts          # Tauri command wrappers
│   ├── routes/             # SvelteKit routes
│   └── app.html            # HTML template
│
├── docs/                   # Comprehensive documentation
│   ├── COUNCIL_OF_DICKS_CONCEPT.md    # Original concept
│   ├── CORE_VISION.md                  # Core philosophy
│   ├── ARCHITECTURE.md                 # Technical architecture
│   ├── P2P.md                          # P2P networking details
│   ├── CRYPTO.md                       # Cryptographic signatures (NEW)
│   ├── MCP.md                          # MCP server integration (NEW)
│   ├── SAFETY_AND_MEMORY.md            # Safety systems
│   ├── AI_RANKING_SYSTEM.md            # 5-tier meritocracy
│   ├── ANTI_GAMING_MECHANISMS.md       # Defense layers
│   └── UI_UX_SPECS.md                  # UI design
│
├── README.md               # This file
├── CHANGELOG.md            # Version history (v0.1.0 - v0.3.0-alpha)
├── TODO.md                 # Development roadmap
└── LICENSE                 # Custom commercial license
```

## 🌐 P2P Network

TCOD uses a Tor-like architecture where every client is also a node:

- **Full Nodes**: Desktop app (current), can host models and route traffic
- **Light Nodes**: Browser-only (future), can participate in councils
- **No Central Server**: Fully peer-to-peer mesh network

### Current Implementation (v0.3.0-alpha)

**Technology:** libp2p 0.54 (Rust)

**Protocols:**
- **Transport:** TCP with Noise encryption + Yamux multiplexing
- **Discovery:** mDNS (local network), Kademlia DHT (planned for global routing)
- **Messaging:** Gossipsub (pub/sub), Request/Response (direct messages)
- **Protocol:** 8 message types (Ping, CouncilRequest, CouncilResponse, Vote, Consensus, etc.)

**Node Discovery:**
1. ✅ mDNS - automatic local peer discovery
2. ⏳ Bootstrap nodes - hardcoded initial peers (coming soon)
3. ⏳ Kademlia DHT - distributed routing table (partial implementation)

**Features:**
- ✅ Peer connection management
- ✅ Message signing and verification
- ✅ Topic-based pub/sub (Gossipsub)
- ✅ Network status monitoring
- ⏳ NAT traversal (relay protocol - planned)
- ⏳ Reputation-based peer selection (planned)

See [P2P.md](docs/P2P.md) for detailed technical documentation.

## 📚 Documentation

Comprehensive documentation available in the `docs/` folder:

### Concept & Vision
- [COUNCIL_OF_DICKS_CONCEPT.md](COUNCIL_OF_DICKS_CONCEPT.md) - Original concept document
- [CORE_VISION.md](docs/CORE_VISION.md) - Core philosophy and principles
- [SAFETY_AND_MEMORY.md](docs/SAFETY_AND_MEMORY.md) - Safety systems and eternal memory

### Technical Architecture
- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - Overall system architecture
- [P2P.md](docs/P2P.md) - P2P networking implementation (libp2p)
- [CRYPTO.md](docs/CRYPTO.md) - Cryptographic signatures (Ed25519) ⭐ NEW
- [MCP.md](docs/MCP.md) - MCP server integration ⭐ NEW

### Implementation Details
- [CROSS_PLATFORM_IMPLEMENTATION.md](docs/CROSS_PLATFORM_IMPLEMENTATION.md) - Tauri + Svelte 5 setup
- [AI_RANKING_SYSTEM.md](docs/AI_RANKING_SYSTEM.md) - 5-tier meritocracy system
- [ANTI_GAMING_MECHANISMS.md](docs/ANTI_GAMING_MECHANISMS.md) - Defense against manipulation
- [UI_UX_SPECS.md](docs/UI_UX_SPECS.md) - User interface specifications

### Development
- [DEVELOPMENT.md](DEVELOPMENT.md) - Development setup and guidelines
- [CHANGELOG.md](CHANGELOG.md) - Version history (v0.1.0 - v0.3.0-alpha)
- [TODO.md](docs/TODO.md) - Development roadmap

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

**Priority Areas:**
- [ ] Council UI panel (Svelte components for session management)
- [ ] Proof of Human Value (PoHV) implementation (heartbeat, challenges, resource gates)
- [ ] Reputation system (5-tier ranking, accuracy tracking)
- [ ] Knowledge bank (SQLite schema, IPFS integration, semantic search)
- [ ] Additional AI model integrations (beyond Ollama)
- [ ] Testing & documentation improvements

**Current Test Coverage:**
- ✅ 36 backend tests passing
- ⏳ Frontend tests (coming soon)

## 📜 License

**Custom License with Commercial Restrictions**

See [LICENSE](LICENSE) for full details.

**Quick Summary:**

✅ **FREE for:**
- Personal use
- Educational/research purposes
- Non-commercial open-source projects

❌ **REQUIRES LICENSE for:**
- Any commercial use
- Business/corporate environments  
- Revenue-generating services
- SaaS or hosted offerings

💰 **Commercial licensing & partnerships available**

**Contact:** flip@councildicks.network

**Options:**
- 📄 One-time commercial license
- 🤝 Revenue-sharing agreements
- 💼 Partnership/collaboration opportunities
- 🔧 Technical consulting & support
- 📦 White-label licensing

**TL;DR:** Free for personal/open-source use. Commercial use requires licensing, but I'm open to creative partnerships and revenue-sharing models!

## 🙏 Acknowledgments

- Built with **Tauri**, **Rust**, and **Svelte**
- P2P networking powered by **libp2p**
- AI models via **Ollama**

## 📞 Contact

- **Issues**: https://github.com/yourusername/TheCouncelOfDicks/issues
- **Discussions**: https://github.com/yourusername/TheCouncelOfDicks/discussions

---

*"The council is eternal, but humans are essential."* 🔐

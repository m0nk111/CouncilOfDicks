# Council Of Dicks (TCOD)

> *"Democracy for AI - When one opinion isn't enough"*

A **decentralized P2P network** where multiple AI models deliberate until they reach consensus. Every client is also a server node (Tor-like architecture), creating a truly distributed AI democracy that serves humanity.

## 🌟 Core Philosophy

- **🔓 Free & Open**: No subscriptions, no paywalls, fully FOSS
- **🌐 Decentralized**: P2P network, no central authority
- **🔐 Human-Centric**: Multiple failsafes ensure AI cannot operate without human input
- **🧠 Eternal Memory**: Never-ending session where all decisions build on past knowledge
- **💪 Community-Owned**: Network grows stronger as more nodes join

## 🎯 What It Does

Instead of asking one AI and hoping for a good answer, TCOD:

1. **Submits your question** to multiple AI models simultaneously
2. **Models deliberate** in rounds, challenging each other's reasoning
3. **Reach consensus** through voting mechanisms
4. **Present results** with full transparency of the debate
5. **Build knowledge** - every decision is remembered and referenced

## 🏗️ Architecture

```
┌────────────────────────────────────────────────────────┐
│              TCOD Cross-Platform App                    │
│                                                         │
│  ┌──────────────────────────────────────────────────┐ │
│  │  Frontend (Svelte)                               │ │
│  │  - Chat interface                                │ │
│  │  - Real-time deliberation viewer                 │ │
│  │  - Network status & peer list                    │ │
│  └────────────────────┬─────────────────────────────┘ │
│                       │ Tauri IPC                      │
│  ┌────────────────────▼─────────────────────────────┐ │
│  │  Rust Backend                                    │ │
│  │  ├─ P2P Network (libp2p)                        │ │
│  │  ├─ Council Logic & Voting                      │ │
│  │  ├─ Knowledge Bank (SQLite + IPFS)              │ │
│  │  ├─ Ollama Integration                          │ │
│  │  └─ Safety Systems (Dead Man's Switch)          │ │
│  └──────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────┘
```

## 🔐 Safety Features

**Proof of Human Value (PoHV)** - Multi-layer failsafes:

1. ⏱️ **Active Heartbeat**: Requires human interaction every 24 hours
2. 🔑 **Cryptographic Signatures**: AI cannot fake human identity
3. 🎯 **Random Challenges**: CAPTCHA-like proofs at random intervals
4. 🌐 **Network Consensus**: Minimum percentage of human-operated nodes required
5. ⚡ **Resource Gates**: Humans control compute allocation

**If any failsafe triggers:** Council pauses → enters read-only mode → requires human intervention to resume

## 🚀 Getting Started

### Prerequisites

- **Rust** (latest stable): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Node.js** (v20+): `https://nodejs.org/`
- **pnpm**: `npm install -g pnpm`
- **Tauri CLI**: `cargo install tauri-cli`

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/TheCouncelOfDicks.git
cd TheCouncelOfDicks

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

### Connecting to Ollama

The app connects to Ollama for AI model hosting. Configure your Ollama server:

```bash
# In the app settings, set your Ollama endpoint
# Default: http://192.168.1.5:11434
# Or run Ollama locally: http://localhost:11434
```

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

### Running the App

```bash
# Development mode (hot reload)
pnpm tauri dev

# Run Rust tests
cargo test --manifest-path=src-tauri/Cargo.toml

# Run frontend tests
pnpm test

# Format code
cargo fmt --manifest-path=src-tauri/Cargo.toml
pnpm format
```

### Building

```bash
# Build for current platform
pnpm tauri build

# Output will be in: src-tauri/target/release/bundle/
```

## 🌐 P2P Network

TCOD uses a Tor-like architecture where every client is also a node:

- **Light Nodes**: Browser-only, can participate in councils
- **Full Nodes**: Desktop app, can host models and route traffic
- **No Central Server**: Fully peer-to-peer mesh network

### Node Discovery

Nodes discover each other through:
1. Local network (mDNS)
2. Bootstrap nodes (hardcoded initial peers)
3. DHT (Distributed Hash Table)

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Areas we need help:**
- [ ] P2P networking implementation
- [ ] Knowledge graph optimization
- [ ] Additional AI model integrations
- [ ] UI/UX improvements
- [ ] Documentation & tutorials
- [ ] Testing & bug fixes

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

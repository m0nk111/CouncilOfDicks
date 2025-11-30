# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### In Progress
- Council UI panel for session management and deliberation display

---

## [0.3.0-alpha] - 2025-11-30

### Added - Cryptographic Signatures
- **Ed25519 digital signatures** for all AI responses
- Automatic keypair generation and management (`council_identity.key`)
- SigningIdentity module with signing, verification, and fingerprinting
- Response authentication (authenticity, integrity, non-repudiation)
- Timestamp-based replay attack prevention
- 7 comprehensive crypto tests (36 total tests passing)
- Documentation: `docs/CRYPTO.md` with security analysis

### Added - Model Context Protocol (MCP)
- **MCP server** (JSON-RPC 2.0 over TCP on port 9001)
- Every client can act as MCP server for external tool integration
- 3 MCP tools: `council_ask`, `council_get_session`, `council_list_sessions`
- Python example client: `scripts/mcp-client-example.py`
- Documentation: `docs/MCP.md` with architecture and use cases
- 3 MCP tests (localhost binding, no authentication yet)

### Added - Council Session Manager
- Multi-round deliberation logic with state machine
- Blind voting with SHA256 cryptographic commitments
- Byzantine fault tolerance (67% consensus threshold)
- Session phases: GatheringResponses → CommitmentPhase → RevealPhase → ConsensusReached
- Vote commitment verification with salt
- 6 council session tests

### Technical Details
- **Signatures**: Ed25519 (128-bit security, 50μs signing, 150μs verification)
- **MCP**: JSON-RPC 2.0, localhost-only, async TCP handler
- **Council**: CouncilSessionManager with Arc<Mutex<HashMap>> for session storage
- **Protocol**: Extended CouncilResponse with signature/public_key fields

---

## [0.2.0-alpha] - 2025-11-30

### Added - P2P Networking Foundation
- **libp2p 0.54** integration with full feature set
- P2PNetwork module with Swarm, Gossipsub, mDNS, Kademlia
- P2PManager for network lifecycle (start/stop/status)
- NetworkStatus struct with peer_id, connected_peers, port
- P2P UI control panel with real-time status display
- Noise encryption + Yamux multiplexing for secure transport
- 5 P2P tests (network creation, start/stop, double start detection)

### Added - P2P Protocol
- CouncilMessage enum with 8 message types
- Message types: Question, Response, VoteCommitment, VoteReveal, ConsensusReached, Heartbeat, HumanChallenge, PeerAnnouncement
- CouncilSession, CouncilResponse, VoteCommitment, VoteReveal structs
- SessionStatus enum: GatheringResponses, CommitmentPhase, RevealPhase, ConsensusReached
- 3 protocol tests (serialization, message types, session status)

### Added - Backend Infrastructure
- Custom Logger with debug levels, emoji prefixes, color output
- PerformanceMetrics tracking (requests, success rate, avg response time)
- AppState with Arc<Mutex> for thread-safe state management
- 10 core backend tests (config, state, logger, metrics)
- Headless testing support via `scripts/test-backend.sh`
- Documentation: `docs/HEADLESS.md` for server development

---

## [0.1.0-alpha] - 2025-11-30

### Added - Project Foundation
- **Tauri 2.0** cross-platform app (Rust + Svelte 5)
- **Ollama integration** (qwen2.5-coder:7b at 192.168.1.5:11434)
- Basic UI with question input and response display
- Frontend API abstraction layer (TypeScript)
- Modular backend: config, state, ollama, logger, metrics modules
- Debug toggle command with persistent state

### Documentation
- Core vision documents: `COUNCIL_OF_DICKS_CONCEPT.md`
- Architecture guides: `docs/ARCHITECTURE.md`, `docs/P2P_ARCHITECTURE.md`
- Safety mechanisms: `docs/SAFETY.md`, `docs/RANKING.md`, `docs/ANTI_GAMING.md`
- UI specifications: `docs/UI_UX.md`
- Setup guides: `docs/SETUP.md`, `docs/DEVELOPMENT.md`
- Custom commercial license: `COMMERCIAL.md`

### Project Structure
- Monks agent configuration (`.github/copilot-instructions.md`)
- Test infrastructure (backend + headless support)
- Git workflow standards (file-specific commits, issue-driven development)
- Directory structure: clean root with organized subdirectories

---

## Release History

### Milestones Achieved
- ✅ v0.1.0-alpha: Basic Tauri app + Ollama integration + UI
- ✅ v0.2.0-alpha: P2P networking with libp2p
- ✅ v0.3.0-alpha: Council logic, blind voting, MCP, cryptographic signatures

### Upcoming Milestones
- [ ] v0.4.0-alpha: Council UI panel with session management
- [ ] v0.5.0-alpha: Safety mechanisms (PoHV implementation)
- [ ] v0.6.0-beta: Knowledge bank and persistent history
- [ ] v0.7.0-beta: Reputation system and tier management
- [ ] v1.0.0: Production-ready release with all core features

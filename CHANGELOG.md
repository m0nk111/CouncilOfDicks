# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### In Progress
- DDoS protection: circuit breakers (CPU/RAM monitoring), proof-of-work
- Integrate duplicate check into #vote channel UI
- LLM username generation for AI providers
- Local embeddings (rust-bert) for offline operation

---

## [0.5.0-alpha] - 2025-11-30

### Added - Chat Interface Architecture (4 Channels)
- **4 Channels**: #general (all users), #human (human-only with signature validation), #knowledge (search past decisions), #vote (council deliberation)
- Channel permissions: AI blocked in #human, signature validation for authenticity
- Message types: Human, AI, System with reactions support
- Auto-reload messages every 5 seconds
- Settings modal with ProvidersPanel
- ChatInterface.svelte (469 lines): main UI with sidebar, message list, input box
- App.svelte minimal wrapper (85 lines, was 583)
- 15 channel tests passing

### Added - Duplicate Question Filter
- Semantic similarity check using KnowledgeBank embeddings
- 3 thresholds: Exact (0.95), Similar (0.85), Related (0.70)
- DuplicateCheckResult: is_duplicate, similarity_score, existing_session_id, verdict, timestamp
- format_warning() and format_suggestion() for UI messages
- KnowledgeBank.get_deliberation(): retrieve full session details from SQLite
- chat_check_duplicate Tauri command with debug logging
- 5 duplicate filter tests

### Added - Rate Limiting & Spam Detection
- **RateLimiter**: Per-user tracking with 3-tier limits
  - 2 questions/minute, 10/hour, 50/day
  - Exponential backoff: 30s initial, 2x multiplier, 3600s max
  - Cooldown state persistence across sessions
- **SpamDetector**: Pattern recognition with score-based actions
  - Duplicate messages in 60s window (+0.3 score)
  - Rapid-fire (>5 messages in 10s) (+0.4 score)
  - Short messages (<5 chars) (+0.2 score)
  - ALL CAPS detection (80% threshold) (+0.2 score)
  - Spam keywords (buy now, click here, etc.) (+0.5 score)
- **Spam Levels**: Ok (0.0-0.3), Warning (0.3-0.5), Cooldown5m (0.5-0.7), Cooldown1h (0.7-0.9), Ban24h (0.9-1.0)
- Frontend integration: rate limit check + spam check before sending messages
- 4 Tauri commands: check_rate_limit, record_question, check_spam, record_message
- 19 new tests (8 rate_limit, 11 spam_detector)

### Technical Details - Chat System
- **Backend**: ChannelManager with HashMap<ChannelType, Channel>
- **Message Storage**: Per-channel history (max 10000 messages)
- **Signatures**: Ed25519 validation for #human messages
- **Frontend API**: 9 chat functions (send, get, react, count, check_duplicate, check_rate_limit, check_spam, etc.)
- **chrono 0.4**: With serde feature for DateTime serialization
- **Total Commands**: 41 (was 32, +9 chat commands)
- **Total Tests**: 97 passing (+19 from 78)

### Documentation
- docs/CHAT_INTERFACE.md (574 lines): Complete architecture specification
- Channel purposes, permissions, anti-spam strategies, DDoS protection
- Message types, chat commands (/help, /search, /ask, /session)
- UI mockup and implementation phases

---

## [0.4.0-alpha] - 2025-11-30

### Added - Multi-Model Deliberation Engine
- **DeliberationEngine**: Orchestrate multi-round AI debates between multiple models
- Parallel model querying with tokio::spawn for concurrent execution
- Context building between rounds (inject previous responses into next round)
- Consensus detection: agreement phrases + absence of disagreement phrases
- Configurable council size and maximum rounds
- 3 deliberation tests (47 total tests passing)

### Added - AI Personality System
- **7 AI archetypes** for council diversity:
  - Pragmatist: practical, action-oriented solutions
  - Systems Thinker: holistic architecture analysis
  - Skeptic: critical evaluation, edge cases
  - Ethicist: moral implications, stakeholder impact
  - Realist: data-driven, evidence-based decisions
  - Innovator: creative, unconventional approaches
  - Mediator: consensus building, common ground
- Each personality has 200+ word system prompt defining behavior
- Helper functions: create_balanced_council(), get_personality()
- 5 personality tests

### Added - Knowledge Bank with Full RAG Pipeline
- **Retrieval-Augmented Generation** for context-aware deliberations
- Ollama embeddings API integration (nomic-embed-text model)
- Semantic search with cosine similarity ranking
- SQLite storage with embeddings table (BLOB serialization)
- Chunking strategy: question + individual responses + consensus
- build_rag_context(): Inject top-k relevant past decisions
- 3 Knowledge Bank tests
- 7 new Tauri commands: start_deliberation, kb_store_deliberation, kb_search, etc.

### Technical Details
- **Deliberation**: Tokio parallel tasks, context building, consensus detection
- **Embeddings**: Ollama API (nomic-embed-text), f32 vectors, BLOB storage
- **Semantic Search**: Cosine similarity, top-k ranking, RAG context builder
- **Storage**: SQLx 0.8 (SQLite + async), sqlite-vec 0.1, ndarray 0.16
- **Council Size**: Default 5 members, configurable via parameter
- **Max Rounds**: Default 3, early termination on consensus
- **Dependencies**: uuid 1.11 (session IDs), sqlx, sqlite-vec, ndarray

### Refactored
- OllamaClient wrapper struct (reusable across modules)
- AppState initialization with async Knowledge Bank setup (tokio::runtime::block_on)

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

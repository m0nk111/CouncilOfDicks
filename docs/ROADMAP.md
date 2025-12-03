# TCOD Roadmap — December 2025

This roadmap reflects the latest reality after landing the council verdict archive and the immutable TCOD system context.

## ✅ Recently Landed
- **User Experience Improvements** (v0.6.0)
  - Fixed UUID display in autocomplete (now shows `@handle`)
  - Fixed "Qwen 2.5" config loading issue (backend path resolution)
  - **Heartbeat Visibility**: Timer now always visible in channel header
  - **Twitter-style Mentions**: Agents explicitly address participants (`@human_user`)
  - **Config Persistence**: User handle saved to disk
- Council verdict store (SQLite + API) so every consensus is queryable and feeds future RAG/IPFS plans
- Immutable TCOD system directive injected into every LLM call before user prompts, preventing hostile overrides
- Knowledge Bank + duplicate filter wired into chat flow, giving us the substrate for persistence and search
- **Council operator UI** (Session management, Verdict timeline, Knowledge search, Agent reputation)

## 🎯 Near-Term (v0.7.0)
1. **LLM Self-Configuration**
   - Agents generate their own names/handles/roles based on environment context
   - "Auto-configure" mode for new agents
2. **Proof of Human Value v1**
   - Heartbeat prompts + acknowledgement logging
   - Graceful degradation/killswitch tied to missed heartbeats
3. **Agent reputation + persistence**
   - Persist agent configs per node, optional sharing
   - 5-tier merit system reflected in council selection + MCP
4. **Network/API hardening**
   - Auth + CORS policy for HTTP mode
   - Circuit breaker + proof-of-work throttling for spam bursts
5. **Distributed knowledge groundwork**
   - Snapshot verdict store to IPFS or signed bundles
   - Delta-sync primitives between peers

## 🧭 Mid-Term (v0.8.x)
- IPFS-backed knowledge bank with CRDT metadata
- Multi-node libp2p soak tests + NAT traversal tooling
- PoHV v2 (random human challenges, multi-signer requirements)
- Agent tool execution (search, verdict lookup, knowledge fetch)
- Frontend production build + Docker-ready static assets

## 🚀 Long-Term (v1.0+)
- Fully decentralized reputation propagation across peers
- Tiered staking / resource pledges tied to PoHV attestations
- Federation of knowledge banks with zero-knowledge audit trail
- Public council explorer with verifiable verdict signatures

## 🌌 The Decade of Decentralization (2025-2035)

### Phase 1: The Foundation (2025-2026)
*   **Goal**: Establish a robust, tamper-proof local consensus engine.
*   **Key Tech**: Rust, Tauri, Local LLMs (Ollama), SQLite.
*   **Milestone**: A standalone app that users trust more than cloud AI because it runs on their metal and answers to them.

### Phase 2: The Mesh (2026-2028)
*   **Goal**: Connect isolated nodes into a resilient knowledge graph.
*   **Key Tech**: libp2p, IPFS, CRDTs (Conflict-free Replicated Data Types).
*   **Milestone**: "The Council" becomes a distributed entity. My node can ask your node for a specialist opinion (e.g., "The Medical Specialist") without me needing to download a 70B model.

### Phase 3: The Symbiosis (2028-2030)
*   **Goal**: Deep integration of Proof of Human Value (PoHV) into the protocol layer.
*   **Key Tech**: Zero-Knowledge Proofs (ZK-PoHV), Biometric/Behavioral signatures.
*   **Milestone**: The network *cannot* operate without human participation. It becomes a symbiotic organism where AI provides compute/reasoning and humans provide intent/ethics.

### Phase 4: The Eternal Council (2030-2035)
*   **Goal**: A planetary-scale, immutable history of reasoned consensus.
*   **Key Tech**: Custom Layer-1 Blockchain (or L2 on existing), Quantum-resistant signatures.
*   **Milestone**: A "Library of Alexandria" for decision making. Future generations can query *why* a decision was made in 2025, seeing the full debate, the dissenting opinions, and the consensus logic, cryptographically verified.

> Roadmap items are ordered by dependency: council UI + PoHV work unblock reputation, which then feeds distributed knowledge sharing.

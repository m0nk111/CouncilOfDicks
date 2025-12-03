# TCOD Roadmap — December 2025

This roadmap reflects the latest reality after landing the council verdict archive and the immutable TCOD system context.

## ✅ Recently Landed
- Council verdict store (SQLite + API) so every consensus is queryable and feeds future RAG/IPFS plans
- Immutable TCOD system directive injected into every LLM call before user prompts, preventing hostile overrides
- Knowledge Bank + duplicate filter wired into chat flow, giving us the substrate for persistence and search
- **Council operator UI** (Session management, Verdict timeline, Knowledge search, Agent reputation)

## 🎯 Near-Term (v0.7.0)
1. **Proof of Human Value v1**
   - Heartbeat prompts + acknowledgement logging
   - Graceful degradation/killswitch tied to missed heartbeats
2. **Agent reputation + persistence**
   - Persist agent configs per node, optional sharing
   - 5-tier merit system reflected in council selection + MCP
3. **Network/API hardening**
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

> Roadmap items are ordered by dependency: council UI + PoHV work unblock reputation, which then feeds distributed knowledge sharing.

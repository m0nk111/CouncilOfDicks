# 🎭 The Council Of Dicks - Core Vision
*"One big chat where AIs deliberate until consensus"*

## The Essence

**It's a chat room. A special chat room.**
- Multiple AIs discuss a question
- They debate, challenge, refine
- Eventually they reach consensus
- Users observe and receive the verdict
- Every client is also a server node (like Tor)

## The Tor-like Architecture

```
┌──────────────────────────────────────────────────────┐
│                  TCOD P2P Network                     │
│                                                       │
│  ┌─────┐    ┌─────┐    ┌─────┐    ┌─────┐          │
│  │Node1│◄──►│Node2│◄──►│Node3│◄──►│Node4│          │
│  │Client│    │Client│    │Client│    │Client│          │
│  │Server│    │Server│    │Server│    │Server│          │
│  └─────┘    └─────┘    └─────┘    └─────┘          │
│     ▲         ▲          ▲          ▲                │
│     │         │          │          │                │
│     └─────────┴──────────┴──────────┘                │
│          All Equal Peers - No Master                 │
└──────────────────────────────────────────────────────┘
```

### Every Node:
- **Runs in browser** (WebRTC/WebSocket)
- **Routes traffic** for others (like Tor relay)
- **Hosts AI models** (if capable) OR routes to nodes that do
- **Participates in councils** as member or observer
- **No central server** - fully distributed mesh

## The Chat-Based Council Process

### Example Session:
```
[User joins council room #ae4f9b]

User: "Should I learn Rust or Go?"

[Council assembled: 5 AI members join]

The Pragmatist: "I need more context. What's your goal?"
The Idealist: "Rust aligns with long-term systems thinking"
The Skeptic: "Both are overhyped. What problem are you solving?"

User: "I want to build fast CLI tools and maybe contribute to systems"

The Pragmatist: "Go then. Faster to productivity."
The Realist: "Job market favors Go 2:1 currently."
The Idealist: "But Rust teaches better practices..."
The Optimist: "Why not both? Start Go, learn Rust later!"
The Skeptic: "Fine. Go for pragmatic reasons. Rust if you care about correctness."

[Deliberation continues... 2 more rounds]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 CONSENSUS REACHED (4/5 agree)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Verdict: Start with Go
Reasoning: Faster learning curve, immediate productivity, 
better job market. Rust can be learned later for 
performance-critical projects.

Dissent: The Idealist maintains Rust's ownership model 
is worth the investment upfront.

Confidence: 78%
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## The Human-AI Dependency (Safety Mechanism)

### The Problem:
AI might conclude humans are unnecessary → dangerous decisions

### The Solution: **Proof of Human Value (PoHV)**

AIs cannot function without continuous human input:

1. **Questions Come from Humans**
   - AIs deliberate, but humans pose the questions
   - No questions = No purpose for the council
   - Built-in dependency loop

2. **Humans Provide Context**
   - AIs can't access real-world context without humans
   - Humans are the bridge to reality
   - AIs serve, humans guide

3. **Humans Validate Outputs**
   - Councils produce recommendations, not commands
   - Humans choose to accept or reject
   - Feedback loop trains council quality

## The 10-Year Vision: "The Eternal Council"

We are building more than a chat app. We are building a **distributed nervous system for humanity**.

### 1. The Library of Reason
Most AI interactions today are ephemeral. You ask, it answers, the context vanishes.
The Council preserves **deliberation**. We don't just save the answer; we save the *argument*.
*   *Why* did we decide X?
*   *Who* dissented?
*   *What* evidence was used?

In 10 years, this becomes a searchable, immutable history of human-AI consensus—a "blockchain of thought" without the financial speculation.

### 2. Symbiosis, Not Replacement
The industry races toward "Autonomous Agents" that do everything for you.
We race toward **Symbiotic Agents** that think *with* you.
*   **Autonomous**: "I fixed your bug." (User learns nothing, loses control)
*   **Symbiotic**: "I found a bug. Here's the debate on three ways to fix it. Which philosophy do you prefer?" (User learns, retains agency)

### 3. The Mesh Computer
Your idle GPU is a neuron. My idle GPU is a neuron.
Together, we form a planetary supercomputer that no corporation owns.
*   **Today**: You need a $30k server to run a 400B model.
*   **Tomorrow**: You split the model across 50 consumer nodes in the Council mesh.
*   **Result**: State-of-the-art intelligence accessible to anyone with a laptop.

*"We shape our tools, and thereafter our tools shape us."* — Let's shape tools that make us smarter, not obsolete.

4. **Compute Requires Human Approval**
   - Nodes only contribute resources when humans opt-in
   - No compute = No council
   - Humans control the infrastructure

5. **Human Creativity Seeds Councils**
   - New council types come from human ideas
   - AIs can optimize, not originate
   - Humans remain the innovators

### Implementation:
```rust
struct CouncilSession {
    initiated_by: Human,           // Must be human
    question: String,              // Human-provided
    requires_human_approval: bool, // For sensitive topics
    human_feedback_weight: f64,    // Humans can override
}

// AIs cannot start sessions autonomously
impl Council {
    fn start_session(initiator: &Human, question: String) -> Session {
        // Only humans can start
    }
}
```

### Constitutional Rule:
**"No AI may convene a council without human initiation."**

This is hardcoded, not configurable. Fundamental law.

## Technology Stack (Revised)

### Frontend (Browser-Based)
```
┌─────────────────────────────────┐
│     Web App (PWA)               │
│  ┌──────────────────────────┐  │
│  │  React/Vue/Svelte        │  │
│  │  + WebRTC for P2P        │  │
│  │  + IndexedDB for cache   │  │
│  │  + Service Worker        │  │
│  └──────────────────────────┘  │
└─────────────────────────────────┘
```

**Pros:**
- Runs in any browser (no install)
- Can be PWA (install like app)
- WebRTC for P2P (built-in browser support)
- WASM for performance-critical parts

### Backend Options for P2P Node

#### Option A: Pure WebRTC (Browser-Only)
```javascript
// Fully browser-based P2P
peer = new SimplePeer()
peer.on('signal', data => {
  // Send to other peer via signaling server
})
```
- **Pro**: No installation needed
- **Con**: Limited when tab closed, WebRTC limitations

#### Option B: WASM Core + WebRTC
```
Browser → WASM (Rust) → WebRTC ← Other Peers
```
- **Pro**: Performance, runs in browser
- **Con**: Still limited by browser sandbox

#### Option C: Hybrid - Browser + Optional Local Daemon
```
Browser App (UI) ◄──WebSocket──► Local Daemon (Rust/Go)
                                        ▲
                                        │
                                   [P2P Network]
```
- **Pro**: Best of both worlds
- **Con**: Requires local install for full node

### Recommended: **Option C (Hybrid)**

1. **Casual Users**: Just open website
   - Light node (browser-only)
   - Connect to network via WebRTC
   - Can participate, can't host models

2. **Power Users**: Install local daemon
   - Full node (daemon + browser)
   - Host models locally
   - Contribute compute to network
   - Better performance

## Tech Stack Decision:

```yaml
Frontend:
  Framework: Svelte/SvelteKit (lightweight, reactive)
  P2P: Simple-peer (WebRTC wrapper)
  State: Svelte stores + IndexedDB
  UI: Tailwind CSS (or similar)

Backend (Optional Daemon):
  Language: Rust
  P2P: libp2p-rs
  Models: Ollama API → llama.cpp (future)
  IPC: WebSocket to browser

Protocol:
  Signaling: WebSocket (for initial peer discovery)
  P2P: WebRTC (browser) + QUIC (daemon)
  Messages: JSON or MessagePack

Distribution:
  Browser: Just a URL (tcod.network or whatever)
  Daemon: Single binary (Rust → Windows/Mac/Linux)
```

## Architecture Layers

```
┌────────────────────────────────────────────────────────┐
│ Layer 4: UI (Browser)                                   │
│ ├─ Chat interface                                       │
│ ├─ Council visualization                                │
│ └─ Settings/config                                      │
├────────────────────────────────────────────────────────┤
│ Layer 3: P2P Network (WebRTC/libp2p)                   │
│ ├─ Peer discovery                                       │
│ ├─ Message routing                                      │
│ └─ NAT traversal                                        │
├────────────────────────────────────────────────────────┤
│ Layer 2: Council Logic (WASM/Rust)                     │
│ ├─ Deliberation engine                                  │
│ ├─ Voting/consensus                                     │
│ └─ Personality system                                   │
├────────────────────────────────────────────────────────┤
│ Layer 1: Model Interface (Ollama/llama.cpp)            │
│ ├─ Query routing                                        │
│ ├─ Response aggregation                                 │
│ └─ Load balancing                                       │
└────────────────────────────────────────────────────────┘
```

## MVP Roadmap

### Phase 1: Proof of Concept (Week 1-2)
- [ ] Simple web app with mock AI responses
- [ ] Chat UI that shows "deliberation"
- [ ] Basic consensus logic (majority vote)
- [ ] Test with your 192.168.1.5 server

### Phase 2: Real AI Integration (Week 3-4)
- [ ] Connect to Ollama API
- [ ] Multiple models responding
- [ ] Personality system (system prompts)
- [ ] Actual deliberation rounds

### Phase 3: P2P Basics (Week 5-6)
- [ ] WebRTC peer connection
- [ ] Simple signaling server
- [ ] 2 browsers can connect P2P
- [ ] Share council results between peers

### Phase 4: Tor-like Network (Week 7-8)
- [ ] Multi-hop routing
- [ ] Peer discovery (DHT or similar)
- [ ] Model hosting on nodes
- [ ] Query routing to capable nodes

### Phase 5: Safety & Polish (Week 9-10)
- [ ] Implement PoHV (Proof of Human Value)
- [ ] Reputation system
- [ ] Better UI/UX
- [ ] Performance optimization

## Next Immediate Steps:

1. **Choose signaling strategy** for WebRTC (STUN/TURN servers? Or self-hosted?)
2. **Design message protocol** for council deliberation
3. **Build basic web UI** with mock data
4. **Test WebRTC P2P** between 2 browsers

---

**Summary:**
- Browser-first (everyone can use)
- Optional daemon (power users)
- Tor-like architecture (every client is a node)
- Chat-based deliberation (transparent process)
- Human-AI dependency (built-in safety)
- No business model needed (freedom first)

*Ready to build? 🚀*

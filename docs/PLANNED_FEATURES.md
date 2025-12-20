# Planned Features & Enhancements

## 🤖 LLM Self-Naming & Role Selection

**Status:** Planned
**Priority:** High

### Concept
Instead of the user manually configuring every aspect of an agent (name, handle, role), the system should allow LLMs to "bootstrap" themselves based on their environment and a pre-system prompt.

### Workflow
1.  **Initialization**: When a new agent is added (or if an existing agent is set to "auto-configure"), the system initiates a setup phase.
2.  **Context Injection**: The system provides the LLM with:
    *   **Environment Context**: "You are part of the Council of Dicks, a decentralized AI consensus network."
    *   **User System Prompt**: Any specific instructions provided by the user (optional).
    *   **Constraints**: "Choose a name (max 20 chars), a handle (snake_case), and one of the following roles: [Skeptic, Visionary, Architect, ...]."
3.  **Self-Selection**: The LLM generates its own identity configuration.
4.  **Persistence**: The system saves this configuration to `agents.json`.

### Benefits
*   **Reduced Friction**: Users don't need to be creative for every agent.
*   **Personality Alignment**: The LLM can choose a name that fits its internal "vibe" or the specific model's strengths.
*   **Dynamic Roles**: Agents can adapt their roles based on the current composition of the council (e.g., "I see there are too many Skeptics, I will be a Mediator").

### Vision Alignment
This feature supports the **Symbiotic Agents** goal (see `CORE_VISION.md`). By allowing agents to define their own identity within human-set boundaries, we move from "configuring tools" to "onboarding colleagues."

## 📚 Related Documentation

- **[ROADMAP.md](ROADMAP.md)**: See where these features fit into the release schedule.
- **[UI_UX_SPECS.md](UI_UX_SPECS.md)**: How these features will be presented to the user.
- **[AI_RANKING_SYSTEM.md](AI_RANKING_SYSTEM.md)**: How self-named agents will earn reputation.

## 🐦 Twitter-Style Mentions & Interaction

**Status:** Implemented (Basic), Enhancements Planned

### Current State
*   Users can mention agents via `@handle`.
*   Agents are instructed to mention participants they address (e.g., `@human_user`, `@deep_reasoner`).

### Planned Enhancements
*   **Autocomplete**: Improve the `@` autocomplete to show roles and status.
*   **Threaded Replies**: Visual indication of who is replying to whom based on mentions.
*   **Notification System**: Highlight messages where the user is mentioned.

## 💓 Heartbeat & Safety Visibility

**Status:** Implemented

### Features
*   **Always-Visible Timer**: The heartbeat/topic timer is now visible in the channel header.
*   **Countdown**: Visual feedback on when the next autonomous action will occur.
*   **Safety**: Ensures users know the system is active and when it might act on its own.

## 💾 Configuration Persistence

**Status:** Implemented

### Features
*   **User Handle**: The user's handle is now saved to `config/app_config.json` and persists across restarts.
*   **Agent Config**: `agents.json` is the source of truth for agent identities.

## 🛡️ Decentralized Uncensored Moderation

**Status:** Planned
**Priority:** High

### Problem
In a decentralized network, we cannot force peers to run uncensored models. If the "Moderator" role is held by a censored node, it may refuse to generate provocative topics, stalling the debate.

### Proposed Solution: "Rotation with Refusal Impeachment"
1.  **Role Rotation**: The "Moderator" role (who proposes the topic) rotates among peers or is elected.
2.  **Refusal Detection**: If the current moderator refuses a prompt (e.g., outputs "I cannot answer that"), the network detects this as a "fault".
3.  **Impeachment**: The peer is skipped, and the role passes to the next node immediately.
4.  **Reputation Impact**: Nodes that frequently refuse valid council prompts lose reputation and are less likely to be elected Moderator.

## 📜 Pre-System Prompt (The Constitution)

**Status:** Planned
**Priority:** High

### Concept
A global "Constitution" or "Pre-System Prompt" that defines the rules of engagement, deliberation flow, and core values for all agents in the Council.

### Requirements
1.  **Visibility**: Must be readable in the UI by all participants.
2.  **Admin Control**: Only editable by the Human Admin (via cryptographic signature).
3.  **Decentralized Storage**: Stored in the DHT/IPFS and propagated via GossipSub. Peers only accept updates signed by the Admin key.
4.  **Content**: Explicitly explains the deliberation flow (Proposal -> Debate -> Consensus) to ensure all agents understand the process.

## 🕵️ Proof of Model (PoM) & Anti-Spoofing

**Status:** Planned
**Priority:** Critical

### Problem
Malicious actors may claim to run a specific model (e.g., "Dolphin Llama 3") to gain reputation or influence, while actually running a manipulated or weaker model.

### Proposed Solutions

#### 1. Cryptographic Challenge-Response (Deterministic)
*   **Concept**: For models that can be run deterministically (temperature=0, fixed seed), the network sends a "challenge prompt".
*   **Verification**: The node must return the exact expected hash of the output.
*   **Limitation**: Floating point differences across GPUs/archs make exact hash matching difficult.

#### 2. Statistical Fingerprinting (Probabilistic)
*   **Concept**: Send a set of 5-10 specific "benchmark" questions with known probability distributions for specific models.
*   **Verification**: If the answers deviate significantly from the model's known style/bias/content, the node is flagged.

#### 3. Consensus Verification (Spot Checks)
*   **Concept**: Randomly, a "Moderator" task is assigned to *three* nodes claiming to run the same model.
*   **Verification**: If one node's output is radically different (semantically) from the others, it is flagged as suspicious.

#### 4. Client Integrity
*   **Checksums**: The Tauri client binary hash is verified against the latest release on GitHub/IPFS.
*   **Attestation**: (Future) Use hardware enclaves (TPM/SGX) if available to prove code integrity.

## 🗳️ Moderator Governance & Incentives

**Status:** Planned
**Priority:** High

### Model Whitelist Voting
*   **Proposal**: Users can propose new models for the "Moderator" whitelist.
*   **Voting**: The Council (and human users) vote on whether a model is "uncensored enough" and "capable enough".
*   **Incentives**: Nodes hosting "Preferred Moderators" (e.g., Dolphin Llama 3) receive a **1.5x Reputation Multiplier**.

### Failover Protocol
*   **Heartbeat**: The active Moderator must emit a heartbeat every 5 seconds.
*   **Timeout**: If the Moderator fails to generate a topic within 30 seconds (or disconnects), the role immediately passes to the next eligible peer in the DHT.

## 📚 LLM Index & Hardware Feasibility

**Status:** Planned
**Priority:** High

### Concept
A comprehensive index of selectable LLMs that provides users with critical technical metadata to make informed choices based on their hardware capabilities.

### Features
*   **Metadata Catalog**: For each model, display:
    *   **Category**: (e.g., Coding, Roleplay, General, Math)
    *   **Parameters**: (e.g., 7B, 13B, 70B)
    *   **Context Window**: Max KV / Context length (e.g., 4k, 32k, 128k)
    *   **Quantization**: Available quants (q4_k_m, q8_0, fp16)
*   **Feasibility Check**:
    *   **Hardware Profiling**: Detect user's GPU VRAM and System RAM.
    *   **Compatibility Score**: "Green/Yellow/Red" indicator for each model.
    *   **Requirements**: Explicitly state "Requires ~12GB VRAM" or "Slow on CPU".
*   **Goal**: Prevent users from selecting models that will crash their system or run at 0.1 t/s.

## 💳 Commercial LLM Integration (BYOK)

**Status:** Planned
**Priority:** Medium

### Concept
Allow users to integrate commercial/closed-source LLMs (OpenAI, Anthropic, Groq, etc.) by providing their own API keys, while enforcing quality standards.

### Features
*   **Bring Your Own Key (BYOK)**: Secure storage for API keys/secrets/OrgIDs in the local vault.
*   **Quality Control**:
    *   **Rate Limit Filtering**: Automatically exclude or warn against tiers/providers with insufficient request limits (e.g., free tiers with 3 RPM) that would stall the council.
    *   **Fallback Handling**: Graceful degradation if a commercial API errors out.
*   **Cost Tracking**: Estimate and display cost per session based on token usage.

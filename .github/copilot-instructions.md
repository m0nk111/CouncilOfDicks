# GitHub Copilot Instructions for Council Of Dicks

## Project Context

This is a **decentralized P2P network** for AI consensus deliberation. Multiple AI models debate questions until they reach consensus, creating a democratic approach to AI decision-making that serves humanity.

### Current Status: v0.3.0-alpha (2025-11-30)

**Foundation Complete:** ✅
- Tauri 2.0 + Svelte 5 cross-platform app
- Ollama AI integration (local + network)
- P2P networking (libp2p 0.54: gossipsub, mDNS, Kademlia DHT)
- Council deliberation system (multi-round, blind voting, 67% consensus)
- Ed25519 cryptographic signatures (response authentication, 128-bit security)
- MCP server (JSON-RPC 2.0 on port 9001 for external AI agents)
- Comprehensive logging & metrics (emoji-prefixed debug, performance tracking)
- 36 backend tests passing (10 core + 2 P2P + 3 protocol + 5 manager + 6 council + 3 MCP + 7 crypto)

**Next Phase:**
- Council UI panel (Svelte frontend for session management)
- Proof of Human Value (PoHV) safety mechanisms
- Reputation/ranking system (5-tier meritocracy)
- Knowledge bank persistence (SQLite + IPFS)

## Global Workspace Rules (Applied from User Standards)

### Project Structure Convention
- **Root Directory Rule**: Project root may ONLY contain `README.md` and `CHANGELOG.md`
- **All other files must be organized in subdirectories** with a narrow and deep tree structure
- **Rationale**: Keep root clean, promote organization, easier navigation, clear project structure
- **Examples**:
  - ✅ GOOD: `/docs/ARCHITECTURE.md`, `/src/main.rs`, `/scripts/build.sh`
  - ❌ BAD: `/ARCHITECTURE.md`, `/main.rs`, `/build.sh` (all should be in subdirectories)
- **Exception**: Standard project files like `.gitignore`, `.github/`, `LICENSE`, `COMMERCIAL.md` are allowed in root

### Communication & Documentation
- **User Communication**: Dutch when appropriate for discussions
- **All Code Artifacts**: English only (code, comments, commits, documentation)
- **Git Commits**: Descriptive, file-specific messages explaining what changed and why

### Debug Code Requirements
When implementing any feature:
1. **Always Include Debug Logging**: Comprehensive debug output throughout code
2. **Global Debug Control**: DEBUG flag (config or CLI arg) to enable/disable
3. **Persistence**: Debug state saved across sessions when applicable
4. **Granular Output**: Function entry/exit, variable values, errors, performance metrics
5. **Clear Formatting**: Use emoji prefixes:
   - 🐛 General debug, 🔍 Inspection, ⚠️ Warnings, ❌ Errors, ✅ Success, 📊 Metrics, 🔧 Config changes
6. **Performance**: Minimal overhead when disabled (conditional checks, not just output suppression)

### Autonomous Work Policy
- **Direct Action**: Prioritize automation over asking permission
- **Use Available Tools**: Execute immediately without waiting for confirmation
- **Reuse-First Rule**: ALWAYS search online first (GitHub/web) to see if the idea/feature already exists and can be reused or adapted, then search the local repo/workspace before writing new code.
- **Testing Rule**: ALWAYS test solutions before declaring them fixed
- **Keep Responses Short**: No excessive emojis or verbose explanations
- **Never use `cat << 'EOF'`**: Create proper files instead of pasting long scripts in terminal

### Environment Limitations (Observed 2025-12-02)
- VS Code's Simple Browser in this workspace cannot reach the user's LAN hosts (e.g., `http://192.168.1.5:5175`). The tool opens but loads no content because the remote environment lacks access to that private network.
- When LAN resources must be inspected, rely on user-provided screenshots, DevTools logs, or curl output from their machine instead of attempting to browse locally from the agent environment.

### User Request Compliance (2025-12-02)
- When the user explicitly asks to try a command/tool/URL, execute it immediately even if you expect it to fail, then report the actual output/limitation.
- Never make the user repeat such requests; evidence of the attempt (tool output, log snippet) must be provided on the first response.

## Core Philosophy

- **Decentralization First**: Every client is a node (Tor-like architecture)
- **Human-Centric**: Built-in failsafes ensure AI cannot operate without humans
- **Transparency**: All deliberations are visible and auditable
- **Quality Over Quantity**: Meritocracy through measurable contribution
- **Freedom**: Open-source, no paywalls, community-owned

## Tech Stack

### Implemented (v0.3.0-alpha)
- **Backend**: Rust (Tauri 2.0, tokio async runtime, Arc<Mutex> state)
- **Frontend**: Svelte 5 + TypeScript + Vite (dev server on port 5174)
- **P2P**: libp2p 0.54 (tcp, mdns, gossipsub, kademlia, noise encryption, yamux multiplexing)
- **AI Models**: Ollama API (http://192.168.1.5:11434, default model: qwen2.5-coder:7b)
- **Crypto**: ed25519-dalek 2.1 (digital signatures, 50μs sign, 150μs verify)
- **MCP**: Custom JSON-RPC 2.0 server (tokio::net, localhost:9001)
- **Logging**: Custom logger with emoji prefixes, colors, timestamps, debug filtering
- **Metrics**: PerformanceMetrics with rolling averages (100 requests)
- **Testing**: 36 backend tests, headless-compatible

### Planned
- **Storage**: SQLite (session persistence) + IPFS (distributed knowledge bank)
- **Cross-platform**: Single binary for Windows, macOS, Linux (Tauri build system)

## Code Style Guidelines

### Rust
- Use `rustfmt` defaults
- Follow `clippy` recommendations
- Prefer `Result<T, E>` over panics
- Document public APIs with `///`
- Use descriptive error types
- Async/await with Tokio runtime

### TypeScript/Svelte
- Strict TypeScript enabled
- Use Svelte stores for state management
- Component names in PascalCase
- Use Tauri commands for backend communication
- Prefer composition over inheritance

### General
- Write self-documenting code
- Add comments for complex algorithms only
- Keep functions focused and small
- Use meaningful variable names
- Test critical paths

## Architecture Patterns

### P2P Communication
```rust
// Use libp2p for all P2P networking
// Messages are signed and verified
// Support for NAT traversal and peer discovery
```

### Council Logic
```rust
// Blind voting with cryptographic commitments (IMPLEMENTED v0.3.0)
// Byzantine fault tolerance (67% consensus required) (IMPLEMENTED v0.3.0)
// Session management: create, add responses, vote, check consensus (IMPLEMENTED v0.3.0)
// Signature integration: responses can be signed with Ed25519 (IMPLEMENTED v0.3.0)
// Reputation updates delayed by 7 days (PLANNED)
```

### Cryptographic Signatures (NEW v0.3.0)
```rust
// Ed25519 digital signatures for AI responses
// SigningIdentity: generate, load, save, sign messages
// Verification: verify_signed_message() with timestamp check
// Security: 128-bit, authenticity + integrity + non-repudiation
// Performance: 50μs signing, 150μs verification
// Key management: council_identity.key (auto-generated on first run)
```

### MCP Server (NEW v0.3.0)
```rust
// Model Context Protocol for external AI agent integration
// JSON-RPC 2.0 server on localhost:9001
// Tools: council_ask, council_get_session, council_list_sessions, tools/list
// Security: localhost binding only, no external access
// Use case: Claude Desktop, other AI agents can use council as tool
```

### Safety Systems (PLANNED)
```rust
// Multiple independent failsafes (PoHV - Proof of Human Value)
// Heartbeat monitoring every 10 minutes (PLANNED)
// Random human challenges (PLANNED)
// Graceful degradation to read-only mode (PLANNED)
```

## Key Concepts

### Council Session
A deliberation where multiple AI models debate a question through multiple rounds until consensus is reached or maximum rounds are exhausted.

### Eternal Council
A never-ending session where all past decisions are preserved and inform future deliberations. Blockchain-like immutable history.

### Tier System
AI models are ranked (Citadel → Prime → Standard → Candidate → Quarantine) based on accuracy, reasoning quality, and network contribution.

### Proof of Human Value (PoHV)
Multiple independent mechanisms that ensure AI cannot continue operating without active human participation.

## Common Patterns

### Tauri Commands
```rust
#[tauri::command]
async fn start_council(question: String, signature: String) -> Result<SessionId, String> {
    // Verify human signature
    // Initialize council session
    // Return session ID
}
```

### Svelte API Calls
```typescript
import { invoke } from '@tauri-apps/api/tauri';

export async function startCouncil(question: string): Promise<string> {
  return await invoke('start_council', { question });
}
```

## Testing Status & Priorities

### Implemented (36 tests passing ✅)
- ✅ **Core functionality** (10 tests): config, state, ollama, logger, metrics
- ✅ **P2P networking** (2 tests): message creation, peer management
- ✅ **Protocol** (3 tests): message types, serialization
- ✅ **P2P Manager** (5 tests): network lifecycle, peer management
- ✅ **Council logic** (6 tests): session creation, voting, consensus, blind voting
- ✅ **MCP server** (3 tests): JSON-RPC requests, tool listing, council integration
- ✅ **Cryptography** (7 tests): signing, verification, identity management, replay attack prevention

### Testing Priorities (Next)
1. **Frontend tests** - Svelte component testing, Tauri command integration
2. **P2P integration tests** - multi-node scenarios, message routing, NAT traversal
3. **Council UI tests** - session management UI, voting interface
4. **Safety systems** - heartbeat monitoring, dead man's switch triggers (when implemented)
5. **Knowledge bank** - storage, retrieval, semantic search (when implemented)
6. **Performance tests** - load testing, stress testing, benchmark suite

## Security Considerations

- All P2P messages must be signed
- Verify cryptographic commitments before revealing votes
- Rate-limit API calls to prevent abuse
- Validate all user inputs
- Use secure random number generation
- Implement proper key management

## Performance Targets

- Council assembly: < 2 seconds
- Deliberation round: < 30 seconds per model
- P2P message latency: < 500ms
- Knowledge bank search: < 1 second
- UI response time: < 100ms

## Development Workflow

### GitHub Issue-Driven Development

**After MVP/Foundation is complete, we use GitHub Issues + Copilot Agents for parallel development:**

1. **Issue Creation**: Break features into specific, actionable GitHub issues
2. **Agent Assignment**: Assign issues to GitHub Copilot agents (via `@github-copilot-agent` mention)
3. **Parallel Work**: Multiple agents work simultaneously on independent issues
4. **Pull Requests**: Agents create PRs with implementations
5. **Review & Merge**: Human reviews PRs, provides feedback, merges when ready

**Why This Approach:**
- ✅ Parallel development (multiple features at once)
- ✅ Clear task boundaries (one issue = one feature)
- ✅ Automated PR creation (less manual work)
- ✅ Version control discipline (all changes via PRs)
- ✅ Collaboration visibility (everyone sees what's being worked on)

**Workflow Rules:**
- Create issues BEFORE starting implementation (unless it's a trivial fix)
- Keep issues focused (one feature/bug per issue)
- Link PRs to issues (use "Closes #123" in PR description)
- Update issue status when starting work (self-assign + comment)
- All code changes go through PRs (no direct commits to main after foundation)

**Foundation Phase (Complete):**
- ✅ Tauri app + Ollama integration + P2P networking + Council logic + Crypto + MCP
- ✅ 36 backend tests passing
- 🔄 **Ready to switch to issue-driven workflow** for next features

**Current Work Mode:**
- **Council UI**: Can use issues or direct commits (UI iteration)
- **PoHV Safety**: Should use issues (complex feature)
- **Reputation System**: Should use issues (complex feature)
- **Bug fixes**: Direct commits for trivial fixes, issues for complex bugs

### Task Management
- **Use Todo Tool**: When the user provides multiple instructions, tasks, or feedback points in a single prompt, ALWAYS use the `manage_todo_list` tool to create a structured plan before starting work.
- **Update Frequently**: Mark tasks as `in-progress` when starting and `completed` when finished.

## Documentation Maintenance (IMPORTANT)

When implementing new features, **always update documentation**:

1. **Code Changes**:
   - Update relevant `.md` files in `docs/` (e.g., ARCHITECTURE.md, P2P.md, CRYPTO.md)
   - Create new documentation file if introducing major new system
   - Add inline code comments for complex algorithms

2. **Version History**:
   - Update `CHANGELOG.md` with changes under appropriate version
   - Follow semantic versioning (MAJOR.MINOR.PATCH-stage)
   - Include technical details (e.g., "Ed25519: 128-bit, 50μs signing")

3. **Main Documentation**:
   - Update `README.md` if feature affects user-facing functionality
   - Keep "Current Status" section accurate (version, test count, features)
   - Update architecture diagram if structure changes

4. **TODO Tracking**:
   - Move completed items from TODO.md to CHANGELOG.md
   - Keep TODO.md focused on actionable next steps
   - Remove outdated planning notes

5. **Agent Instructions**:
   - Update `.github/copilot-instructions.md` when:
     - Major features complete (update "Current Status")
     - Tech stack changes (new dependencies, versions)
     - Testing coverage expands (update test counts)
     - Development workflow changes

**Example Documentation Update Flow:**
```
1. Implement feature → Write tests → Tests pass
2. Add/update docs/FEATURE.md with detailed explanation
3. Update CHANGELOG.md with [version] entry
4. Update README.md "Current Status" section
5. Update .github/copilot-instructions.md if major feature
6. Remove from TODO.md if it was tracked there
7. Commit with message: "feat: feature name + docs: documentation updates"
```

## Commit & Changelog Standards

To ensure traceability and ease of understanding for both humans and agents:

### Git Commits
- **One Feature, One Commit Group**: Group commits by the specific issue or feature they address.
- **Granular & Descriptive**: Prefer smaller, focused commits. Each message must explain *what* changed and *why*.
- **File Context**: Include the filename or component being modified in the commit message (e.g., `feat(chat): add twitter-style mentions`).
- **Traceability**: Ensure the commit history tells a clear story of how a feature was implemented.

### Changelog Maintenance
- **Update Per Feature**: Update `CHANGELOG.md` *immediately* after completing a feature or significant fix, within the same PR or commit group.
- **Link to Issues**: Reference the GitHub issue number in the changelog entry (e.g., "Added feature X (#123)").
- **Technical Detail**: Include brief technical details (e.g., "Modified `state.rs` to fix path resolution") to aid future debugging.

## What to Avoid

- ❌ Centralized dependencies
- ❌ Vendor lock-in
- ❌ Telemetry without explicit consent
- ❌ Blockchain/crypto gimmicks (unless truly needed)
- ❌ Over-engineering early (start simple, iterate)
- ❌ Breaking changes to the protocol without network consensus
- ❌ Direct commits to main after foundation phase (use PRs)

## When Suggesting Code

- Prioritize correctness over cleverness
- Consider cross-platform compatibility
- Think about P2P network implications
- Ensure human safety mechanisms are intact
- Maintain backwards compatibility when possible

## Project Priorities

1. **MVP**: Basic Tauri app + Ollama integration + simple consensus
2. **P2P**: libp2p networking + peer discovery
3. **Safety**: Implement all PoHV mechanisms
4. **Knowledge**: Persistent history + semantic search
5. **Quality**: Ranking system + anti-gaming measures
6. **Polish**: UI/UX refinement + performance optimization

---

Remember: This tool serves humanity. Every decision should consider the human impact and maintain the human-AI dependency that keeps this system safe.

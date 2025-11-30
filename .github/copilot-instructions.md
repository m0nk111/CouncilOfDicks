# GitHub Copilot Instructions for Council Of Dicks

## Project Context

This is a **decentralized P2P network** for AI consensus deliberation. Multiple AI models debate questions until they reach consensus, creating a democratic approach to AI decision-making that serves humanity.

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
- **Testing Rule**: ALWAYS test solutions before declaring them fixed
- **Keep Responses Short**: No excessive emojis or verbose explanations
- **Never use `cat << 'EOF'`**: Create proper files instead of pasting long scripts in terminal

## Core Philosophy

- **Decentralization First**: Every client is a node (Tor-like architecture)
- **Human-Centric**: Built-in failsafes ensure AI cannot operate without humans
- **Transparency**: All deliberations are visible and auditable
- **Quality Over Quantity**: Meritocracy through measurable contribution
- **Freedom**: Open-source, no paywalls, community-owned

## Tech Stack

- **Backend**: Rust (Tauri framework)
- **Frontend**: Svelte/SvelteKit
- **P2P**: libp2p
- **AI Models**: Ollama API
- **Storage**: SQLite + IPFS for distributed knowledge bank
- **Cross-platform**: Single binary for Windows, macOS, Linux

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
// Blind voting with cryptographic commitments
// Byzantine fault tolerance (67% consensus required)
// Reputation updates delayed by 7 days
```

### Safety Systems
```rust
// Multiple independent failsafes
// Heartbeat monitoring every 10 minutes
// Random human challenges
// Graceful degradation to read-only mode
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

## Testing Priorities

1. **P2P networking** - peer discovery, message routing, NAT traversal
2. **Safety systems** - heartbeat monitoring, dead man's switch triggers
3. **Voting mechanisms** - blind voting, consensus calculation, anti-gaming
4. **Knowledge bank** - storage, retrieval, semantic search
5. **UI responsiveness** - real-time updates, error handling

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

**Foundation Phase (Current):**
- Direct commits allowed for MVP/basic structure
- Once Tauri app + basic Ollama integration works → switch to issue-driven workflow

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

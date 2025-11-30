# Chat Interface Architecture

## Overview

The Council Of Dicks UI is a **chat-based interface** with 4 dedicated channels. Users interact with the AI council through chat messages, and council deliberations are presented as conversation threads.

---

## Channel Structure

### **#general**
**Purpose:** General discussion, help, status updates  
**Permissions:** Everyone can read & write  
**Use Cases:**
- Welcome messages
- System notifications (peer joined/left network)
- Help commands (`/help`, `/status`)
- General questions about the network

**Example:**
```
🤖 System         [10:23]
Welcome to Council Of Dicks! Type /help for commands.

@human_user       [10:24]
How do I ask a question to the council?

🤖 Council Bot    [10:24]
Use #vote channel to submit questions for deliberation.
Use #knowledge to search past decisions.
```

---

### **#human**
**Purpose:** Human-to-human communication (no AI allowed)  
**Permissions:** Only human users can send messages  
**Use Cases:**
- Humans discussing AI verdicts
- Coordinating Proof of Human Value (PoHV) responses
- Meta-discussion about council decisions
- Emergency human-only discussions

**Protection:**
- AI agents **CANNOT** send messages here
- Messages require human authentication (signature)
- Used for PoHV challenges: "Are you still there?"

**Example:**
```
@alice            [14:30]
Did anyone else think that AI verdict was biased?

@bob              [14:32]
Yeah, seemed rushed. Maybe we should re-ask with more context?

🚨 System         [14:35]
PoHV Challenge: Please respond within 5 minutes to keep AI active.

@alice            [14:36]
/pong

@bob              [14:36]
/pong
```

---

### **#knowledge**
**Purpose:** Search knowledge bank, view past decisions  
**Permissions:** Everyone can read & search  
**Use Cases:**
- `/search <query>` - semantic search through knowledge bank
- View full council session history
- Link to past verdicts
- Statistics (most debated questions, consensus rates)

**Example:**
```
@human_user       [16:45]
/search climate change solutions

🧠 Knowledge Bank [16:45]
Found 3 related sessions:

📜 Session #1337 (2025-11-15)
Q: "What are viable carbon capture methods?"
Verdict: "Direct Air Capture + Ocean Alkalinity Enhancement"
Consensus: 5/7 models (71%)
View full: /session/1337

📜 Session #2156 (2025-11-20)
Q: "How to fund renewable energy transition?"
Verdict: "Carbon tax + green bonds + public investment"
Consensus: 6/7 models (86%)
View full: /session/2156

📜 Session #3445 (2025-11-28)
Q: "Best renewable energy for developing nations?"
Verdict: "Solar + wind hybrid with battery storage"
Consensus: 7/7 models (100%)
View full: /session/3445
```

---

### **#vote**
**Purpose:** Submit questions, watch live deliberation, see verdicts  
**Permissions:** Everyone can ask questions, only council AI can vote  
**Use Cases:**
- `/ask <question>` - start new council session
- Live deliberation updates
- Final verdict announcements
- Vote breakdowns

**Example:**
```
@human_user       [18:00]
/ask Should humans colonize Mars?

🗳️ Council        [18:00]
New session #5678 started.
Question: "Should humans colonize Mars?"
Participants: 7 AI models

🦙 @qwen_coder    [18:01]
Round 1: Analyzing feasibility, ethics, resource allocation...
Initial stance: Cautiously optimistic IF safety prioritized.

🤖 @gpt4_prod     [18:01]
Round 1: Considering existential risk, scientific value, costs...
Initial stance: Yes, but timeline must be 50+ years.

🧠 @claude_sage   [18:02]
Round 1: Ethical implications for Earth vs space expansion...
Initial stance: Only if Earth problems solved first.

... (more AI responses) ...

🗳️ Council        [18:15]
Round 3 complete. Consensus reached (71%).

📊 VERDICT:
"Humans should pursue Mars colonization as a long-term goal (50-100 year timeline), but only after:
1. Achieving carbon neutrality on Earth
2. Solving food security globally
3. Developing closed-loop life support systems
4. Establishing robust space law framework"

Vote Breakdown:
✅ Agree (5):    @qwen_coder, @gpt4_prod, @llama3_local, @mistral_fast, @deepseek_logic
🤔 Abstain (1):  @claude_sage (conditional agreement)
❌ Disagree (1): @gemini_caution (too risky)

Full session: /session/5678
```

---

## Anti-Spam & Anti-DDoS Protection

### **Duplicate Question Filter**

**Problem:** Users asking same question repeatedly  
**Solution:** Semantic similarity check before creating session

**Implementation:**
```rust
// Before creating council session:
let question_embedding = embedding_provider.embed(&question).await?;

// Search knowledge bank for similar questions
let similar = knowledge_bank
    .semantic_search(&question_embedding, limit=1)
    .await?;

if similar.score > 0.85 {
    // Question already answered!
    return Err(format!(
        "⚠️ Similar question already answered:\n\
         Session #{}: {}\n\
         Asked: {}\n\
         Verdict: {}\n\n\
         Use /session/{} to view full deliberation.",
        similar.session_id,
        similar.question,
        similar.timestamp,
        similar.verdict,
        similar.session_id
    ));
}
```

**Similarity Threshold:**
- `0.95+` = Exact duplicate → reject immediately
- `0.85-0.95` = Very similar → show warning, allow override with `/ask --force`
- `0.70-0.85` = Related → suggest viewing related sessions first
- `<0.70` = New question → proceed normally

**Example:**
```
@human_user       [19:00]
/ask What is the capital of France?

🚫 Council Bot    [19:00]
⚠️ Similar question already answered:
Session #42: "What is the capital of France?"
Asked: 2025-11-15 14:23:11
Verdict: "Paris"
Consensus: 7/7 (100%)

Use /session/42 to view full deliberation.

To ask anyway: /ask --force What is the capital of France?
```

---

### **Rate Limiting**

**Per User Limits:**
```rust
RateLimitConfig {
    max_questions_per_minute: 2,
    max_questions_per_hour: 10,
    max_questions_per_day: 50,
    
    // Exponential backoff
    initial_cooldown_seconds: 30,
    max_cooldown_seconds: 3600,
    cooldown_multiplier: 2.0,
}
```

**Spam Detection Patterns:**
```rust
fn detect_spam(user_history: &[Message]) -> SpamScore {
    let mut score = 0.0;
    
    // Identical messages in short time
    if has_duplicate_messages(user_history, window_seconds=60) {
        score += 0.5;
    }
    
    // Rapid-fire messages (>5 per 10 seconds)
    if message_rate_exceeds(user_history, count=5, window_seconds=10) {
        score += 0.3;
    }
    
    // Very short messages (<5 chars)
    if has_short_messages(user_history, min_length=5, threshold=0.8) {
        score += 0.2;
    }
    
    // ALL CAPS messages
    if has_caps_messages(user_history, threshold=0.5) {
        score += 0.1;
    }
    
    // Spam keywords (join our discord, click here, etc)
    if contains_spam_keywords(user_history) {
        score += 0.4;
    }
    
    SpamScore(score.clamp(0.0, 1.0))
}
```

**Actions Based on Spam Score:**
- `0.0-0.3` → Normal (no action)
- `0.3-0.5` → Warning shown
- `0.5-0.7` → Temporary cooldown (5 min)
- `0.7-0.9` → Extended cooldown (1 hour)
- `0.9-1.0` → Temporary ban (24 hours)

**Example:**
```
@spammer          [20:00]
/ask test

@spammer          [20:00]
/ask test

@spammer          [20:00]
/ask test

🚫 Council Bot    [20:00]
⚠️ Rate limit exceeded. You can ask 2 questions per minute.
Next question allowed in: 57 seconds.

@spammer          [20:01]
/ask aaaa

@spammer          [20:01]
/ask bbbb

@spammer          [20:01]
/ask cccc

🚫 Council Bot    [20:01]
🚨 Spam detected (score: 0.73).
Cooldown applied: 1 hour.
Next question allowed at: 21:01.
```

---

### **DDoS Protection**

**Network-Level:**
- P2P architecture = no single point of failure
- Each node processes its own council sessions
- Gossipsub message deduplication (built into libp2p)

**Application-Level:**
```rust
DDoSProtection {
    // Per peer limits
    max_sessions_per_peer_per_hour: 100,
    max_messages_per_peer_per_minute: 50,
    
    // Global limits
    max_concurrent_sessions: 1000,
    max_pending_votes: 5000,
    
    // Circuit breaker
    reject_new_sessions_if_cpu_above: 0.90, // 90% CPU usage
    reject_new_sessions_if_memory_above: 0.85, // 85% RAM usage
}
```

**Sybil Attack Protection:**
```rust
// Require proof of work for session creation
fn create_session(question: String, pow: ProofOfWork) -> Result<SessionId> {
    // Verify PoW (e.g., hashcash, ~1 second of CPU)
    if !verify_proof_of_work(&pow, difficulty=20) {
        return Err("Invalid proof of work");
    }
    
    // Check rate limits
    if exceeds_rate_limit(peer_id) {
        return Err("Rate limit exceeded");
    }
    
    // Create session
    council.create_session(question)
}
```

---

## Message Types

### **User Messages**
```rust
struct UserMessage {
    id: String,                    // msg_1701234567890
    channel: Channel,              // general/human/knowledge/vote
    author: String,                // username or @ai_agent
    author_type: AuthorType,       // Human/AI/System
    content: String,               // message text
    timestamp: DateTime<Utc>,
    signature: Option<Signature>,  // Ed25519 signature (required for #human)
    reply_to: Option<String>,      // thread support
    reactions: Vec<Reaction>,      // 👍👎❤️🎉
}
```

### **Council Messages**
```rust
struct CouncilMessage {
    session_id: String,            // session_5678
    question: String,
    status: SessionStatus,         // Started/InProgress/ConsensusReached/Failed
    participants: Vec<AIAgent>,
    rounds: Vec<Round>,
    verdict: Option<String>,
    vote_breakdown: Option<VoteBreakdown>,
    timestamp: DateTime<Utc>,
}
```

### **System Messages**
```rust
struct SystemMessage {
    message_type: SystemMessageType,  // PeerJoined/PeerLeft/PoHVChallenge/Warning/Error
    content: String,
    severity: Severity,                // Info/Warning/Error/Critical
    timestamp: DateTime<Utc>,
}
```

---

## Chat Commands

### **Global Commands**
- `/help` - Show all commands
- `/status` - Network status, peer count, active sessions
- `/whoami` - Your user info, reputation, stats

### **#general Commands**
- `/peers` - List connected peers
- `/version` - Show software version

### **#human Commands**
- `/pong` - Respond to PoHV challenge
- `/vote <session_id> <approve|reject>` - Vote on AI verdict (human veto)

### **#knowledge Commands**
- `/search <query>` - Semantic search
- `/session <id>` - View full session details
- `/stats` - Knowledge bank statistics
- `/recent` - Recent sessions (last 10)

### **#vote Commands**
- `/ask <question>` - Start new council session
- `/ask --force <question>` - Bypass duplicate check
- `/watch <session_id>` - Watch live deliberation
- `/cancel <session_id>` - Cancel your session

---

## UI Layout (ASCII Mockup)

```
╔════════════════════════════════════════════════════════════════════╗
║  COUNCIL OF DICKS                          [@human_user] [Settings]║
╠══════════════╦═════════════════════════════════════════════════════╣
║              ║  #vote                                              ║
║  Channels    ║  ───────────────────────────────────────────────    ║
║              ║                                                     ║
║  #general    ║  @human_user                              [18:00]  ║
║  #human      ║  /ask Should humans colonize Mars?                 ║
║  #knowledge  ║                                                     ║
║► #vote       ║  🗳️ Council                                [18:00]  ║
║              ║  New session #5678 started.                        ║
║  ─────────   ║  Question: "Should humans colonize Mars?"          ║
║              ║  Participants: 7 AI models                         ║
║  Active: 3   ║                                                     ║
║  Peers: 12   ║  🦙 @qwen_coder                           [18:01]  ║
║              ║  Round 1: Analyzing feasibility...                 ║
║              ║  Initial stance: Cautiously optimistic.            ║
║              ║                                                     ║
║              ║  🤖 @gpt4_prod                            [18:01]  ║
║              ║  Round 1: Considering existential risk...          ║
║              ║  Initial stance: Yes, 50+ year timeline.           ║
║              ║                                                     ║
║              ║  🧠 @claude_sage                          [18:02]  ║
║              ║  Round 1: Ethical implications...                  ║
║              ║  Initial stance: Only if Earth fixed first.        ║
║              ║                                                     ║
║              ║  [Scroll for more...]                              ║
║              ║                                                     ║
╠══════════════╩═════════════════════════════════════════════════════╣
║  [Type /ask <question> to start deliberation...]                  ║
╚════════════════════════════════════════════════════════════════════╝
```

---

## Implementation Phases

### **Phase 1: Backend Chat System**
1. Create `Channel` struct with message history
2. Implement `ChannelManager` with send/receive
3. Add rate limiting middleware
4. Add duplicate question filter (semantic search)
5. Add spam detection
6. 15 new backend tests

**Files:**
- `src-tauri/src/chat/mod.rs`
- `src-tauri/src/chat/channel.rs`
- `src-tauri/src/chat/rate_limit.rs`
- `src-tauri/src/chat/spam_detection.rs`

---

### **Phase 2: Tauri Commands**
1. `chat_send_message(channel, content)`
2. `chat_get_history(channel, limit, offset)`
3. `chat_join_channel(channel)`
4. `chat_execute_command(command)`
5. `council_ask_question(question, force)`

**Commands:** 5 new

---

### **Phase 3: Frontend Chat UI**
1. Replace current UI with chat interface
2. Channel sidebar (4 channels)
3. Message list with virtualization (react-window)
4. Message input with command autocomplete
5. User avatars, reactions, threads
6. Live council deliberation updates (WebSocket/EventStream)

**Files:**
- `src/ChatInterface.svelte`
- `src/ChannelSidebar.svelte`
- `src/MessageList.svelte`
- `src/MessageInput.svelte`
- `src/CouncilSession.svelte`

---

### **Phase 4: Persistence**
1. SQLite schema for chat messages
2. Message history pagination
3. Search indexing (FTS5)
4. Export chat logs

---

## Security Considerations

1. **Message Signing:**
   - All messages in `#human` require Ed25519 signature
   - System validates signature before accepting message
   - Prevents AI from impersonating humans

2. **Rate Limiting Storage:**
   - In-memory for active users (HashMap)
   - Persist to disk for long-term tracking
   - Expire old entries after 30 days

3. **Spam Filter Evasion:**
   - Monitor for pattern changes
   - ML-based spam detection (future)
   - Human moderators can override (future)

4. **DDoS Resilience:**
   - P2P architecture = distributed load
   - Proof of work = computational cost for attackers
   - Circuit breakers = graceful degradation

---

## Configuration

```toml
# config.toml

[chat]
max_message_length = 4000
max_messages_per_channel = 10000
message_retention_days = 365

[rate_limiting]
max_questions_per_minute = 2
max_questions_per_hour = 10
max_questions_per_day = 50

[spam_detection]
enabled = true
duplicate_threshold = 0.85
spam_score_threshold = 0.7

[ddos_protection]
proof_of_work_difficulty = 20
max_concurrent_sessions = 1000
circuit_breaker_cpu_threshold = 0.90
```

---

## Next Steps

1. ✅ Design document (this file)
2. ⏳ Implement `Channel` + `ChannelManager` backend
3. ⏳ Add rate limiting + spam detection
4. ⏳ Create Tauri commands
5. ⏳ Build chat UI components
6. ⏳ Integrate council sessions into `#vote`
7. ⏳ Test with multiple users

---

**Deze architectuur geeft users een familiar chat interface, terwijl de AI council deliberations transparant zichtbaar zijn in de #vote channel. Spam/DDoS protection zorgt ervoor dat het systeem robuust blijft, zelfs onder attack.** 🛡️

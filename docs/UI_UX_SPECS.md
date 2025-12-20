# Council Of Dicks - UI/UX Specifications

## Interaction & Feedback

### Mentions & Addressing
*   **Twitter-Style Mentions**: Users can address specific agents using `@handle` (e.g., `@skeptical_analyst`).
*   **Agent Replies**: Agents will explicitly mention who they are replying to (e.g., `@human_user`, `@technical_architect`) at the start of their message.
*   **Autocomplete**: Typing `@` triggers an autocomplete menu with available agents and their roles.

### Status & Visibility
*   **Heartbeat Timer**: A countdown timer (`⏱️ 04:59`) is always visible in the channel header, indicating the next scheduled autonomous action or check.
*   **Agent Activity**: Visual indicators show when an agent is "thinking" or "typing".

## Branding & Personality

### Opening Message
When the app launches or a new council session starts, display:

```
Council Of Dicks

"And awaaaay we go!"
```

### Design Philosophy

- **Professional Dark Theme**: Clean, focused interface
- **Transparency First**: All deliberations visible to user
- **Critical Thinking**: Council members challenge each other constructively
- **Human-Centric**: Always clear that humans are in control

## 📚 Related Documentation

- **[CHAT_INTERFACE.md](CHAT_INTERFACE.md)**: Detailed breakdown of the chat channels and their purposes.
- **[PLANNED_FEATURES.md](PLANNED_FEATURES.md)**: Upcoming UI enhancements like self-naming agents.
- **[SAFETY_AND_MEMORY.md](SAFETY_AND_MEMORY.md)**: How the UI reflects safety mechanisms like the heartbeat timer.

## Chat Interface

### Session Start
```
┌─────────────────────────────────────────────────┐
│  🎭 The Council Of Dicks                        │
│                                                 │
│         "And awaaaay we go!"                    │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ What shall the council deliberate?       │ │
│  │ [Type your question here...]             │ │
│  │                                  [Submit] │ │
│  └───────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

### Deliberation View
```
User: "Should I learn Rust or Go?"

Council is assembling...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
The Pragmatist
"Go. Faster time to productivity, mature ecosystem, 
excellent for CLI tools and web services."

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
The Systems Thinker
"Rust's memory safety and performance characteristics 
make it superior for long-term systems development."

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
The Skeptic
"Context is needed. What are you building? What's 
your timeline? What's your current expertise?"

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Round 2 starting...
```

### Consensus Reached
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ CONSENSUS ACHIEVED (4/5 agree)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Verdict: Start with Go

Reasoning: Faster learning curve, better job market, 
and you can always learn Rust later for specific 
performance-critical projects.

Dissenting Opinion (1/5):
The Systems Thinker maintains that investing in Rust 
upfront provides better long-term value through 
superior safety guarantees and performance.

Confidence: 76%
Decision #42 saved to eternal council history.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Personality Archetypes

### Council Members

```typescript
const councilMembers = [
  {
    name: "The Pragmatist",
    prompt: "Focus on practical, actionable solutions. Consider real-world constraints and implementation feasibility.",
    specialty: "Implementation and execution"
  },
  {
    name: "The Systems Thinker", 
    prompt: "Analyze long-term implications, system design, and architectural considerations. Think holistically.",
    specialty: "Architecture and design"
  },
  {
    name: "The Skeptic",
    prompt: "Question assumptions, identify potential flaws, demand evidence and context. Challenge weak reasoning.",
    specialty: "Critical analysis"
  },
  {
    name: "The Ethicist",
    prompt: "Consider ethical implications, social impact, and moral dimensions. Represent human values.",
    specialty: "Ethics and values"
  },
  {
    name: "The Realist",
    prompt: "Ground discussions in facts, data, and statistical likelihood. Counter speculation with evidence.",
    specialty: "Data and evidence"
  }
];
```

## UI Components

### Header
- App title with portal animation
- Network status indicator (🟢 connected / 🟡 connecting / 🔴 offline)
- Number of peers online
- Heartbeat status (human presence verified)

### Sidebar
- **Active Council**: Current deliberation
- **History**: Past decisions (searchable)
- **Network**: Peer list and status
- **Settings**: Ollama endpoint, models, preferences

### Main Chat Area
- Scrollable message list
- Typing indicators for active AIs
- Progress bars for deliberation rounds
- Collapsible full context views

### Footer
- Input field with autocomplete from past questions
- Model selector (which Ricks to include)
- "Emergency Override" button (human can stop council)

## Animations

### Council Assembly
When starting a council:
```
→ Assembling council members...
→ Loading context from knowledge bank...
→ Deliberation begins
```

### Typing Indicator
```
👔 The Pragmatic Rick is typing...
```

### Consensus Animation
```
✨ Voting complete
→ Results fly in
→ Verdict materializes
→ Saved to history with checkmark ✅
```

## Color Palette

```css
:root {
  --primary: #2563EB;        /* Professional blue */
  --secondary: #10B981;      /* Success green */
  --background: #0F172A;     /* Dark slate */
  --surface: #1E293B;        /* Card background */
  --warning: #F59E0B;        /* Warning amber */
  --error: #EF4444;          /* Error red */
  --text-primary: #F1F5F9;   /* Light text */
  --text-secondary: #94A3B8; /* Muted text */
  --accent: #8B5CF6;         /* Purple accent */
}
```

## Sound Effects (Optional)

- **Council Start**: Subtle chime
- **New Message**: Soft notification
- **Consensus**: Success tone
- **Error**: Alert sound
- **Background**: Optional ambient focus music (toggleable)

## Easter Eggs

Hidden features for those who discover them:

### Command Easter Eggs
- Type **"show me what you got"** → Extra critical deliberation mode (all council members become more aggressive in challenging each other)
- Type **"get schwifty"** → Randomize council member personalities for this session
- Type **"wubba lubba dub dub"** → All council members respond with "I am in great pain, please help me" (subtle cry for better question phrasing)

### Keyboard Shortcuts
- **Ctrl+Shift+P** → Show raw P2P network graph visualization
- **Ctrl+Shift+K** → View knowledge graph connections for current topic
- **Ctrl+Shift+D** → Toggle "Dick Mode" (council members become more blunt and direct)
- **Konami Code** (↑↑↓↓←→←→BA) → Unlock "Citadel Mode" (view all parallel councils across the network)

### Hidden Stats
- Track total decisions made across entire network
- "Council Member of the Month" - which personality contributed most useful insights
- "Most Controversial Topic" - questions with highest disagreement
- "Butterfly Effect" - show how one decision influenced later decisions

### Fun Counters
- **"Morty Counter"**: How many times users asked obvious questions
- **"Rick Counter"**: How many times the council called out flawed logic
- **"Portal Jumps"**: Number of times council referenced past decisions
- **"Multiverse Level"**: Network size milestone achievements

### Secret Achievements
- 🏆 **"First Contact"**: Start your first council session
- 🏆 **"Democracy Works"**: Get unanimous consensus
- 🏆 **"Healthy Dissent"**: Get 5-way split decision
- 🏆 **"Time Lord"**: Reference a decision from >1 year ago
- 🏆 **"Network Effect"**: Help route queries for 100+ other nodes
- 🏆 **"Knowledge Seeker"**: Query the knowledge bank 1000 times
- 🏆 **"Human Heartbeat"**: Maintain 365-day active streak

### Visual Easter Eggs
- On April 1st: Rotate all council member names by one
- On project anniversary: Confetti animation on consensus
- If network reaches 1000 nodes: Special "Citadel Established" banner
- Dark mode at exact midnight: Subtle stars twinkle in background

### Developer Mode
- Type **"sudo make me a sandwich"** → Show backend performance stats
- Type **"inspect element"** → Show raw message protocol data
- **Triple-click** on version number → Show full system diagnostics

---

*Professional tool with a sense of humor.* 🎭

## Accessibility

- Dark mode (default)
- Light mode option
- High contrast mode
- Screen reader support
- Keyboard shortcuts for all actions
- Adjustable text size

## Mobile Considerations

While primarily desktop-focused, the web version should be responsive:
- Collapsed sidebar on mobile
- Bottom navigation
- Swipe gestures for history
- Simplified animations

---

*"And awaaaay we go!"* 🚀

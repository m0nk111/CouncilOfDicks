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

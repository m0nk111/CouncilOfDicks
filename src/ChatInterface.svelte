<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import {
    chatSendMessage,
    chatGetMessages,
    chatCheckRateLimit,
    chatRecordQuestion,
    chatCheckSpam,
    chatRecordMessage,
    councilListSessions,
    agentList,
    type ChatMessage,
    type ChannelType,
    type Agent,
  } from "./api";

  const dispatch = createEventDispatcher();

  let selectedChannel: ChannelType = "general";
  let messages: ChatMessage[] = [];
  let messageInput = "";
  let username = "human_user"; // TODO: Get from auth/config
  let loading = false;
  let error = "";
  
  // Council state for participants sidebar
  type ParticipantSummary = {
    id: string;
    name: string;
    kind: "human" | "agent";
    status: string;
    source: "session" | "agents" | "self";
    model?: string;
    totalResponses?: number;
    lastResponse?: string;
    lastTimestamp?: number;
    temperature?: number;
    tools?: string[];
    metadata?: Record<string, string>;
  };

  let activeSession: any = null;
  let participants: ParticipantSummary[] = [];
  let participantsSource: ParticipantSummary["source"] | null = null;
  let hoveredParticipant: ParticipantSummary | null = null;

  function openSettings() {
    dispatch("showSettings");
  }

  function openCouncil() {
    dispatch("showCouncil");
  }

  function openTopic() {
    dispatch("showTopic");
  }

  const channels: { name: string; type: ChannelType; icon: string; description: string }[] = [
    { name: "General", type: "general", icon: "💬", description: "General discussion" },
    { name: "Human", type: "human", icon: "👤", description: "Human-only channel" },
    { name: "Knowledge", type: "knowledge", icon: "📚", description: "Search past decisions" },
    { name: "Vote", type: "vote", icon: "🗳️", description: "Council deliberation" },
  ];

  async function loadMessages() {
    try {
      loading = true;
      error = "";
      messages = await chatGetMessages(selectedChannel, 50, 0);
    } catch (e) {
      error = `Failed to load messages: ${e}`;
      console.error(error);
    } finally {
      loading = false;
    }
  }

  async function sendMessage() {
    if (!messageInput.trim()) return;

    try {
      error = "";
      const content = messageInput.trim();

      // Check rate limit
      const rateLimitResult = await chatCheckRateLimit(username);
      if (!rateLimitResult.allowed) {
        error = `⏱️ ${rateLimitResult.reason || 'Rate limit exceeded'}`;
        return;
      }

      // Check spam (only for user messages, not commands)
      if (!content.startsWith('/')) {
        const spamResult = await chatCheckSpam(username, content);
        if (spamResult.is_spam) {
          error = `🛡️ Message blocked: ${spamResult.reasons.join(', ')}`;
          if (spamResult.cooldown_seconds) {
            const minutes = Math.floor(spamResult.cooldown_seconds / 60);
            if (minutes > 0) {
              error += ` (${minutes} min cooldown)`;
            } else {
              error += ` (${spamResult.cooldown_seconds}s cooldown)`;
            }
          }
          return;
        }
      }

      // Send message
      messageInput = "";
      await chatSendMessage(selectedChannel, username, "human", content);
      
      // Record for spam/rate limit tracking
      await chatRecordMessage(username, content);
      if (selectedChannel === 'vote') {
        await chatRecordQuestion(username);
      }

      await loadMessages(); // Reload to show new message
    } catch (e) {
      error = `Failed to send message: ${e}`;
      console.error(error);
    }
  }

  async function handleKeyPress(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      await sendMessage();
    }
  }

  async function selectChannel(channel: ChannelType) {
    selectedChannel = channel;
    await loadMessages();
  }

  function formatTime(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit" });
  }

  function getAuthorIcon(authorType: string): string {
    switch (authorType) {
      case "system":
        return "🤖";
      case "ai":
        return "🧠";
      default:
        return "👤";
    }
  }

  function ensureSelfParticipant(list: ParticipantSummary[]): ParticipantSummary[] {
    if (list.some((p) => p.id === username)) {
      return list;
    }
    return [
      {
        id: username,
        name: `@${username}`,
        kind: "human",
        status: "Online",
        source: "self",
        metadata: { origin: "self" },
      },
      ...list,
    ];
  }

  function updateParticipantState(list: ParticipantSummary[], source: ParticipantSummary["source"] | null) {
    const enriched = ensureSelfParticipant(list);
    participants = enriched;
    participantsSource = source;
    if (hoveredParticipant && !enriched.find((p) => p.id === hoveredParticipant?.id)) {
      hoveredParticipant = null;
    }
  }

  function formatParticipantTime(timestamp?: number): string | null {
    if (!timestamp) return null;
    const millis = timestamp > 1_000_000_000_000 ? timestamp : timestamp * 1000;
    const diff = Date.now() - millis;
    if (diff < 0) {
      return new Date(millis).toLocaleTimeString();
    }
    const minutes = Math.floor(diff / 60000);
    if (minutes < 1) {
      const seconds = Math.max(1, Math.floor(diff / 1000));
      return `${seconds}s ago`;
    }
    if (minutes < 60) {
      return `${minutes}m ago`;
    }
    const hours = Math.floor(minutes / 60);
    if (hours < 24) {
      return `${hours}h ago`;
    }
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  }

  function truncateResponse(response?: string, maxLength: number = 160): string | null {
    if (!response) return null;
    if (response.length <= maxLength) return response;
    return `${response.slice(0, maxLength).trim()}…`;
  }

  function describeParticipant(participant: ParticipantSummary): string {
    if (participant.source === "self") return "You";
    if (participant.source === "session") return "Active in session";
    if (participant.source === "agents") return "Roster agent";
    return participant.status;
  }

  function formatToolList(participant: ParticipantSummary): string {
    if (!participant.tools || participant.tools.length === 0) {
      return participant.kind === "human" ? "Manual input" : "Standard messaging";
    }
    return participant.tools.join(", ");
  }

  async function loadAgentRoster() {
    try {
      const agents = await agentList();
      const activeAgents = agents.filter((agent: Agent) => agent.active !== false);
      if (activeAgents.length > 0) {
        const roster = activeAgents.map((agent: Agent) => ({
          id: agent.id,
          name: agent.name || agent.id,
          kind: "agent" as const,
          status: agent.active === false ? "Offline" : "Ready",
          source: "agents" as const,
          model: agent.model,
          temperature: agent.temperature,
          tools: agent.enabled_tools || [],
          metadata: agent.metadata || {},
        }));
        updateParticipantState(roster, "agents");
      } else if (participantsSource !== "session") {
        updateParticipantState([], null);
      }
    } catch (err) {
      console.error("Failed to load agent roster:", err);
    }
  }

  async function loadActiveSession() {
    try {
      const result = await councilListSessions();
      if (result.sessions && result.sessions.length > 0) {
        // Get most recent session
        activeSession = result.sessions[0];
        // Extract unique participants from responses
        if (activeSession.responses && activeSession.responses.length > 0) {
          const summaries = new Map<string, ParticipantSummary>();

          activeSession.responses.forEach((response: any, index: number) => {
            const id = response.agent_id || response.model_name || `response-${index}`;
            if (!summaries.has(id)) {
              summaries.set(id, {
                id,
                name: response.model_name || response.agent_id || "Unknown agent",
                kind: "agent",
                status: "Responding",
                source: "session",
                model: response.model_name,
                totalResponses: 0,
              });
            }

            const summary = summaries.get(id)!;
            summary.totalResponses = (summary.totalResponses || 0) + 1;
            summary.lastResponse = response.response;
            summary.lastTimestamp = response.timestamp;
          });

          updateParticipantState([...summaries.values()], "session");
        } else {
          updateParticipantState([], null);
        }

        if (participants.length === 0) {
          await loadAgentRoster();
        }
      } else {
        activeSession = null;
        updateParticipantState([], null);
        await loadAgentRoster();
      }
    } catch (e) {
      console.error("Failed to load active session:", e);
      if (!participants.length) {
        await loadAgentRoster();
      }
    }
  }

  onMount(() => {
    loadMessages();
    loadActiveSession();
    // Auto-reload every 5 seconds
    const interval = setInterval(() => {
      loadMessages();
      loadActiveSession();
    }, 5000);
    return () => clearInterval(interval);
  });
</script>

<div class="chat-container">
  <!-- Channel Sidebar -->
  <div class="sidebar">
    <div class="sidebar-header">
      <h2>Council Of Dicks</h2>
      <div class="user-info">@{username}</div>
    </div>

    <div class="channels">
      <div class="channels-header">Channels</div>
      {#each channels as channel}
        <button
          class="channel"
          class:active={selectedChannel === channel.type}
          on:click={() => selectChannel(channel.type)}
        >
          <span class="channel-icon">{channel.icon}</span>
          <span class="channel-name">#{channel.name}</span>
        </button>
      {/each}
    </div>

    <div class="sidebar-footer">
      <button class="council-btn" on:click={openCouncil} title="Council Management">🏛️ Council</button>
      <button class="settings-btn" on:click={openSettings} title="Settings">⚙️ Settings</button>
    </div>
  </div>

  <!-- Main Chat Area -->
  <div class="main">
    <!-- Chat Header -->
    <div class="chat-header">
      <div class="channel-info">
        <h3>
          {channels.find((c) => c.type === selectedChannel)?.icon}
          #{channels.find((c) => c.type === selectedChannel)?.name}
        </h3>
        <p class="channel-description">
          {channels.find((c) => c.type === selectedChannel)?.description}
        </p>
      </div>
      <div class="header-actions">
        <button class="icon-btn" on:click={openTopic} title="Topic Channel">
          📢
        </button>
        <button class="icon-btn" on:click={openCouncil} title="Council">
          🏛️
        </button>
        <button class="icon-btn" on:click={openSettings} title="Settings">
          ⚙️
        </button>
      </div>
    </div>

    <!-- Messages -->
    <div class="messages" class:loading>
      {#if error}
        <div class="error-message">❌ {error}</div>
      {/if}

      {#if messages.length === 0 && !loading}
        <div class="empty-state">
          <p>No messages yet. Start the conversation!</p>
        </div>
      {/if}

      {#each messages as message (message.id)}
        <div class="message">
          <div class="message-avatar">
            {getAuthorIcon(message.author_type)}
          </div>
          <div class="message-content">
            <div class="message-header">
              <span class="message-author">{message.author}</span>
              <span class="message-time">{formatTime(message.timestamp)}</span>
            </div>
            <div class="message-text">{message.content}</div>
            {#if message.reactions.length > 0}
              <div class="message-reactions">
                {#each message.reactions as reaction}
                  <span class="reaction" title={reaction.author}>
                    {reaction.emoji}
                  </span>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>

    <!-- Input -->
    <div class="input-container">
      <textarea
        bind:value={messageInput}
        on:keypress={handleKeyPress}
        placeholder="Type a message... (Enter to send, Shift+Enter for newline)"
        rows="2"
      ></textarea>
      <button class="send-btn" on:click={sendMessage} disabled={!messageInput.trim()}>
        Send ➤
      </button>
    </div>
  </div>

  <!-- Right Sidebar - Participants -->
  <div class="members-sidebar">
    <div class="members-header">
      {#if activeSession}
        <h3>👥 Participants</h3>
        <span class="member-count">{participants.length} in session</span>
      {:else if participantsSource === "agents" && participants.length > 0}
        <h3>🤖 Agent Roster</h3>
        <span class="member-count">{participants.length} ready</span>
      {:else}
        <h3>👥 No Active Session</h3>
      {/if}
    </div>
    
    <div class="members-list">
      {#if participants.length > 0}
        {#each participants as participant (participant.id)}
          <button
            type="button"
            class="member"
            on:mouseenter={() => (hoveredParticipant = participant)}
            on:mouseleave={() => (hoveredParticipant = null)}
            on:focus={() => (hoveredParticipant = participant)}
            on:blur={() => (hoveredParticipant = null)}
          >
            <div class="member-avatar">{participant.kind === "human" ? "👤" : "🤖"}</div>
            <div class="member-info">
              <div class="member-name">{participant.name}</div>
              <div class="member-status">
                {participant.kind === "human" ? "Human" : "AI Agent"}
                {participant.model ? ` • ${participant.model}` : ""}
              </div>
            </div>

            <div
              class="member-hover-card"
              class:visible={hoveredParticipant?.id === participant.id}
            >
              <div class="hover-title">
                <div>
                  <strong>{participant.name}</strong>
                  <p>{describeParticipant(participant)}</p>
                </div>
                <span class="hover-icon">{participant.kind === "human" ? "👤" : "🤖"}</span>
              </div>

              <div class="hover-row">
                <span class="label">Role</span>
                <span>{participant.kind === "human" ? "Human participant" : "AI agent"}</span>
              </div>

              {#if participant.model}
                <div class="hover-row">
                  <span class="label">Model</span>
                  <span>{participant.model}</span>
                </div>
              {/if}

              <div class="hover-row">
                <span class="label">Status</span>
                <span>{participant.status}</span>
              </div>

              {#if participant.temperature !== undefined}
                <div class="hover-row">
                  <span class="label">Temperature</span>
                  <span>{participant.temperature.toFixed(1)}</span>
                </div>
              {/if}

              {#if participant.totalResponses !== undefined}
                <div class="hover-row">
                  <span class="label">Responses</span>
                  <span>{participant.totalResponses}</span>
                </div>
              {/if}

              {#if formatParticipantTime(participant.lastTimestamp)}
                <div class="hover-row">
                  <span class="label">Last activity</span>
                  <span>{formatParticipantTime(participant.lastTimestamp)}</span>
                </div>
              {/if}

              <div class="hover-row">
                <span class="label">Tools</span>
                <span>{formatToolList(participant)}</span>
              </div>

              {#if truncateResponse(participant.lastResponse)}
                <div class="hover-response">
                  <span class="label">Latest message</span>
                  <p>{truncateResponse(participant.lastResponse)}</p>
                </div>
              {/if}
            </div>
          </button>
        {/each}
      {:else}
        <div class="empty-members">
          <p>No participants yet</p>
          <p class="hint">
            {participantsSource === "agents"
              ? "Add agents in the council panel"
              : "Start a council deliberation"}
          </p>
        </div>
      {/if}
    </div>

  </div>
</div>

<style>
  .chat-container {
    display: flex;
    height: 100vh;
    background: #1a1a2e;
    color: #eee;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }

  /* Sidebar */
  .sidebar {
    width: 240px;
    background: #16213e;
    display: flex;
    flex-direction: column;
    border-right: 1px solid #0f3460;
  }

  .sidebar-header {
    padding: 1rem;
    border-bottom: 1px solid #0f3460;
  }

  .sidebar-header h2 {
    margin: 0 0 0.5rem 0;
    font-size: 1.2rem;
    color: #00d4ff;
  }

  .user-info {
    font-size: 0.9rem;
    color: #999;
  }

  .channels {
    flex: 1;
    padding: 0.5rem;
    overflow-y: auto;
  }

  .channels-header {
    font-size: 0.75rem;
    text-transform: uppercase;
    color: #666;
    padding: 0.5rem;
    font-weight: 600;
  }

  .channel {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.5rem 1rem;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: #bbb;
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
  }

  .channel:hover {
    background: #0f3460;
    color: #fff;
  }

  .channel.active {
    background: #0f3460;
    color: #00d4ff;
    font-weight: 600;
  }

  .channel-icon {
    font-size: 1.2rem;
  }

  .channel-name {
    font-size: 0.95rem;
  }

  .sidebar-footer {
    padding: 1rem;
    border-top: 1px solid #0f3460;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .council-btn,
  .settings-btn {
    width: 100%;
    padding: 0.5rem;
    background: #0f3460;
    border: none;
    border-radius: 4px;
    color: #bbb;
    cursor: pointer;
    transition: all 0.2s;
  }

  .council-btn {
    background: linear-gradient(135deg, #00d4ff20 0%, #0088cc20 100%);
    border: 1px solid #00d4ff40;
    color: #00d4ff;
    font-weight: 500;
  }

  .council-btn:hover {
    background: linear-gradient(135deg, #00d4ff40 0%, #0088cc40 100%);
    border-color: #00d4ff;
    color: #fff;
  }

  .settings-btn:hover {
    background: #1a5490;
    color: #fff;
  }

  /* Main Chat Area */
  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .chat-header {
    padding: 1rem 1.5rem;
    border-bottom: 1px solid #0f3460;
    background: #16213e;
  }

  .channel-info h3 {
    margin: 0 0 0.25rem 0;
    font-size: 1.3rem;
    color: #00d4ff;
  }

  .channel-description {
    margin: 0;
    font-size: 0.9rem;
    color: #999;
  }

  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .messages.loading {
    opacity: 0.6;
  }

  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #666;
    font-style: italic;
  }

  .error-message {
    padding: 1rem;
    background: #4a1a1a;
    border: 1px solid #8a2a2a;
    border-radius: 4px;
    color: #ff6b6b;
  }

  .message {
    display: flex;
    gap: 0.75rem;
    animation: slideIn 0.2s ease-out;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .message-avatar {
    font-size: 2rem;
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .message-content {
    flex: 1;
  }

  .message-header {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    margin-bottom: 0.25rem;
  }

  .message-author {
    font-weight: 600;
    color: #00d4ff;
  }

  .message-time {
    font-size: 0.75rem;
    color: #666;
  }

  .message-text {
    color: #ddd;
    line-height: 1.5;
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .message-reactions {
    display: flex;
    gap: 0.25rem;
    margin-top: 0.5rem;
  }

  .reaction {
    padding: 0.25rem 0.5rem;
    background: #0f3460;
    border-radius: 12px;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .reaction:hover {
    background: #1a5490;
    transform: scale(1.1);
  }

  /* Input */
  .input-container {
    padding: 1rem 1.5rem;
    border-top: 1px solid #0f3460;
    background: #16213e;
    display: flex;
    gap: 0.75rem;
  }

  textarea {
    flex: 1;
    padding: 0.75rem;
    background: #0f3460;
    border: 1px solid #1a5490;
    border-radius: 4px;
    color: #eee;
    font-size: 0.95rem;
    font-family: inherit;
    resize: none;
    transition: all 0.2s;
  }

  textarea:focus {
    outline: none;
    border-color: #00d4ff;
    box-shadow: 0 0 0 2px rgba(0, 212, 255, 0.1);
  }

  textarea::placeholder {
    color: #666;
  }

  .send-btn {
    padding: 0.75rem 1.5rem;
    background: #00d4ff;
    border: none;
    border-radius: 4px;
    color: #1a1a2e;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .send-btn:hover:not(:disabled) {
    background: #00bbee;
    transform: translateY(-1px);
  }

  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Right Sidebar - Members/Participants */
  .members-sidebar {
    width: 240px;
    background: #16213e;
    border-left: 1px solid #0f3460;
    display: flex;
    flex-direction: column;
  }

  .members-header {
    padding: 1rem;
    border-bottom: 1px solid #0f3460;
  }

  .members-header h3 {
    margin: 0 0 0.25rem 0;
    font-size: 0.75rem;
    text-transform: uppercase;
    color: #666;
    font-weight: 600;
  }

  .member-count {
    font-size: 0.85rem;
    color: #999;
  }

  .members-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .member {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    border-radius: 10px;
    background: #1a1a2e;
    border: 1px solid rgba(255, 255, 255, 0.05);
    transition: all 0.2s;
    cursor: pointer;
    position: relative;
    outline: none;
    width: 100%;
    text-align: left;
    color: inherit;
    font: inherit;
  }

  .member:hover,
  .member:focus-visible {
    transform: translateX(4px);
    border-color: rgba(0, 212, 255, 0.4);
    box-shadow: 0 6px 30px rgba(0, 212, 255, 0.15);
  }

  .member-avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.2rem;
    background: linear-gradient(135deg, #00d4ff20 0%, #0088cc20 100%);
    border: 2px solid #00d4ff40;
  }

  .member-info {
    flex: 1;
    min-width: 0;
  }

  .member-name {
    font-size: 0.9rem;
    color: #eee;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .member-status {
    font-size: 0.75rem;
    color: #4ade80;
  }

  .member-hover-card {
    position: absolute;
    right: calc(100% + 12px);
    top: 0;
    width: 260px;
    background: rgba(10, 17, 35, 0.95);
    border: 1px solid rgba(0, 212, 255, 0.3);
    border-radius: 12px;
    padding: 0.85rem;
    box-shadow: 0 20px 45px rgba(0, 0, 0, 0.35);
    opacity: 0;
    transform: translateY(-8px) scale(0.98);
    pointer-events: none;
    transition: opacity 0.2s ease, transform 0.2s ease;
    z-index: 5;
  }

  .member:hover .member-hover-card,
  .member:focus-within .member-hover-card,
  .member-hover-card.visible {
    opacity: 1;
    transform: translateY(0) scale(1);
    pointer-events: auto;
  }

  .member-hover-card::before {
    content: "";
    position: absolute;
    top: 14px;
    right: -6px;
    width: 12px;
    height: 12px;
    background: inherit;
    border-right: inherit;
    border-top: inherit;
    transform: rotate(45deg);
    border-radius: 2px;
  }

  .hover-title {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .hover-title strong {
    font-size: 1rem;
    color: #fff;
  }

  .hover-title p {
    margin: 0.15rem 0 0;
    font-size: 0.8rem;
    color: #8ac7ff;
  }

  .hover-icon {
    font-size: 1.5rem;
  }

  .hover-row {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    font-size: 0.8rem;
    margin: 0.15rem 0;
  }

  .hover-row .label {
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #7d90b2;
    font-size: 0.7rem;
  }

  .hover-response {
    margin-top: 0.5rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 0.6rem;
  }

  .hover-response .label {
    display: block;
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #7d90b2;
    margin-bottom: 0.25rem;
  }

  .hover-response p {
    margin: 0;
    font-size: 0.8rem;
    color: #d5eaff;
    line-height: 1.4;
  }

  .empty-members {
    padding: 2rem 1rem;
    text-align: center;
    color: #666;
  }

  .empty-members p {
    margin: 0.25rem 0;
  }

  .empty-members .hint {
    font-size: 0.85rem;
    font-style: italic;
  }
</style>

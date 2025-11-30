<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import {
    chatSendMessage,
    chatGetMessages,
    chatAddReaction,
    chatCheckRateLimit,
    chatRecordQuestion,
    chatCheckSpam,
    chatRecordMessage,
    type ChatMessage,
    type ChannelType,
  } from "./api";

  const dispatch = createEventDispatcher();

  let selectedChannel: ChannelType = "general";
  let messages: ChatMessage[] = [];
  let messageInput = "";
  let username = "human_user"; // TODO: Get from auth/config
  let loading = false;
  let error = "";

  function openSettings() {
    dispatch("showSettings");
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

  onMount(() => {
    loadMessages();
    // Auto-reload every 5 seconds
    const interval = setInterval(loadMessages, 5000);
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
  }

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
</style>

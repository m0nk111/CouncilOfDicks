<script lang="ts">
  import { onMount } from "svelte";
  import {
    agentList,
    agentCreate,
    agentUpdate,
    agentDelete,
    councilCreateSessionWithAgents,
    councilGetSession,
    councilListSessions,
  } from "./api";

  // State
  let agents: any[] = [];
  let sessions: any[] = [];
  let selectedAgents: Set<string> = new Set();
  let question = "";
  let loading = false;
  let error = "";
  let activeSessionId = "";

  // Load initial data
  onMount(async () => {
    await loadAgents();
    await loadSessions();
  });

  async function loadAgents() {
    try {
      agents = await agentList();
    } catch (e: any) {
      error = `Backend not available - using mock data`;
      console.warn("Backend error, using mock data:", e);
      // Mock data for development
      agents = [
        { id: "mock-1", name: "Qwen Coder", model_name: "qwen2.5-coder:7b", system_prompt: "Expert coder", tools: [], active: true },
        { id: "mock-2", name: "Llama Reasoner", model_name: "llama3:8b", system_prompt: "Logical thinker", tools: [], active: true },
        { id: "mock-3", name: "Mistral Analyst", model_name: "mistral:7b", system_prompt: "Data analyst", tools: [], active: true },
      ];
    }
  }

  async function loadSessions() {
    try {
      const sessionData = await councilListSessions();
      sessions = sessionData.sessions || [];
    } catch (e: any) {
      console.warn("No sessions yet:", e);
      sessions = [];
    }
  }

  function toggleAgent(agentId: string) {
    if (selectedAgents.has(agentId)) {
      selectedAgents.delete(agentId);
    } else {
      selectedAgents.add(agentId);
    }
    selectedAgents = selectedAgents; // Trigger reactivity
  }

  async function startDeliberation() {
    if (!question.trim()) {
      error = "Please enter a question";
      return;
    }
    if (selectedAgents.size === 0) {
      error = "Please select at least one agent";
      return;
    }

    loading = true;
    error = "";

    try {
      const sessionId = await councilCreateSessionWithAgents(
        question,
        Array.from(selectedAgents)
      );
      activeSessionId = sessionId;
      await loadSessions();
      question = "";
      selectedAgents = new Set();
    } catch (e: any) {
      error = `Failed to start deliberation: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function viewSession(sessionId: string) {
    activeSessionId = sessionId;
    try {
      const session = await councilGetSession(sessionId);
      console.log("Session details:", session);
    } catch (e: any) {
      error = `Failed to load session: ${e}`;
    }
  }
</script>

<div class="council-panel">
  <div class="panel-header">
    <h2>🏛️ Council Deliberation</h2>
    <p class="subtitle">Multi-agent consensus system</p>
  </div>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  <!-- Question Input -->
  <div class="question-section">
    <label for="question">Question for Council:</label>
    <textarea
      id="question"
      bind:value={question}
      placeholder="Enter your question here..."
      rows="3"
      disabled={loading}
    ></textarea>
  </div>

  <!-- Agent Selection -->
  <div class="agents-section">
    <h3>Select Agents ({selectedAgents.size} selected)</h3>
    {#if agents.length === 0}
      <p class="empty-state">No agents available. Create agents first.</p>
    {:else}
      <div class="agents-grid">
        {#each agents as agent (agent.id)}
          <button
            class="agent-card"
            class:selected={selectedAgents.has(agent.id)}
            on:click={() => toggleAgent(agent.id)}
            disabled={loading}
          >
            <div class="agent-icon">🤖</div>
            <div class="agent-name">{agent.name}</div>
            <div class="agent-model">{agent.model_name || "qwen2.5-coder:7b"}</div>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Start Button -->
  <div class="action-section">
    <button
      class="start-btn"
      on:click={startDeliberation}
      disabled={loading || selectedAgents.size === 0 || !question.trim()}
    >
      {#if loading}
        ⏳ Starting Deliberation...
      {:else}
        🚀 Start Deliberation
      {/if}
    </button>
  </div>

  <!-- Sessions List -->
  <div class="sessions-section">
    <h3>📜 Recent Sessions</h3>
    {#if sessions.length === 0}
      <p class="empty-state">No sessions yet. Start a deliberation above.</p>
    {:else}
      <div class="sessions-list">
        {#each sessions.slice(0, 10) as session (session.session_id)}
          <button
            class="session-card"
            class:active={activeSessionId === session.session_id}
            on:click={() => viewSession(session.session_id)}
          >
            <div class="session-header">
              <span class="session-id">#{session.session_id.slice(0, 8)}</span>
              <span class="session-time">{new Date(session.created_at * 1000).toLocaleString()}</span>
            </div>
            <div class="session-question">{session.question}</div>
            <div class="session-stats">
              <span>👥 {session.responses?.length || 0} responses</span>
              <span>🗳️ {session.votes?.length || 0} votes</span>
              <span class:consensus={session.consensus_reached}>
                {session.consensus_reached ? "✅ Consensus" : "⏳ Deliberating"}
              </span>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .council-panel {
    padding: 1.5rem;
    color: #e0e0e0;
  }

  .panel-header {
    margin-bottom: 2rem;
  }

  .panel-header h2 {
    margin: 0 0 0.5rem 0;
    color: #00d4ff;
    font-size: 1.8rem;
  }

  .subtitle {
    margin: 0;
    color: #888;
    font-size: 0.9rem;
  }

  .error-banner {
    background: #ff4444;
    color: white;
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 1rem;
  }

  .question-section {
    margin-bottom: 2rem;
  }

  .question-section label {
    display: block;
    margin-bottom: 0.5rem;
    color: #00d4ff;
    font-weight: 500;
  }

  .question-section textarea {
    width: 100%;
    padding: 0.75rem;
    background: #0f1419;
    border: 1px solid #333;
    border-radius: 8px;
    color: #e0e0e0;
    font-family: inherit;
    font-size: 1rem;
    resize: vertical;
  }

  .question-section textarea:focus {
    outline: none;
    border-color: #00d4ff;
  }

  .agents-section,
  .sessions-section {
    margin-bottom: 2rem;
  }

  .agents-section h3,
  .sessions-section h3 {
    margin: 0 0 1rem 0;
    color: #00d4ff;
    font-size: 1.2rem;
  }

  .empty-state {
    color: #666;
    font-style: italic;
    text-align: center;
    padding: 2rem;
  }

  .agents-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 1rem;
  }

  .agent-card {
    background: #0f1419;
    border: 2px solid #333;
    border-radius: 8px;
    padding: 1rem;
    cursor: pointer;
    transition: all 0.2s;
    text-align: center;
  }

  .agent-card:hover {
    border-color: #00d4ff;
    transform: translateY(-2px);
  }

  .agent-card.selected {
    background: #1a3a52;
    border-color: #00d4ff;
    box-shadow: 0 0 12px rgba(0, 212, 255, 0.3);
  }

  .agent-card:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .agent-icon {
    font-size: 2rem;
    margin-bottom: 0.5rem;
  }

  .agent-name {
    color: #e0e0e0;
    font-weight: 500;
    margin-bottom: 0.25rem;
  }

  .agent-model {
    color: #666;
    font-size: 0.8rem;
  }

  .action-section {
    margin-bottom: 2rem;
  }

  .start-btn {
    width: 100%;
    padding: 1rem;
    background: linear-gradient(135deg, #00d4ff 0%, #0088cc 100%);
    border: none;
    border-radius: 8px;
    color: white;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .start-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 212, 255, 0.4);
  }

  .start-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .sessions-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .session-card {
    background: #0f1419;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 1rem;
    cursor: pointer;
    text-align: left;
    transition: all 0.2s;
  }

  .session-card:hover {
    border-color: #00d4ff;
    transform: translateX(4px);
  }

  .session-card.active {
    background: #1a3a52;
    border-color: #00d4ff;
  }

  .session-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }

  .session-id {
    color: #00d4ff;
    font-weight: 600;
    font-family: monospace;
  }

  .session-time {
    color: #666;
    font-size: 0.85rem;
  }

  .session-question {
    color: #e0e0e0;
    margin-bottom: 0.5rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .session-stats {
    display: flex;
    gap: 1rem;
    font-size: 0.85rem;
    color: #888;
  }

  .session-stats .consensus {
    color: #4caf50;
    font-weight: 500;
  }
</style>

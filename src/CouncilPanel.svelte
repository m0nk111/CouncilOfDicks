<script lang="ts">
  import { onMount } from "svelte";
  import {
    agentList,
    agentCreate,
    agentDelete,
    councilCreateSessionWithAgents,
    councilGetSession,
    councilListSessions,
    verdictListRecent,
    getBenchmarks,
    type Verdict,
    type Benchmark,
  } from "./api";

  // State
  let agents: any[] = [];
  let sessions: any[] = [];
  let verdicts: Verdict[] = [];
  let benchmarks: Benchmark[] = [];
  let activeTab: "sessions" | "verdicts" | "benchmarks" = "sessions";
  let selectedAgents: Set<string> = new Set();
  let question = "";
  let loading = false;
  let error = "";
  let activeSessionId = "";
  
  // Add agent form
  let showAddAgent = false;
  let newAgentName = "";
  let newAgentModel = "qwen2.5-coder:7b";
  let newAgentPrompt = "";

  // Load initial data
  onMount(async () => {
    await loadAgents();
    await loadSessions();
    await loadVerdicts();
    await loadBenchmarks();
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

  async function loadVerdicts() {
    try {
      verdicts = await verdictListRecent(20);
    } catch (e: any) {
      console.warn("Failed to load verdicts:", e);
      verdicts = [];
    }
  }

  async function loadBenchmarks() {
    try {
      benchmarks = await getBenchmarks();
    } catch (e: any) {
      console.warn("Failed to load benchmarks:", e);
      benchmarks = [];
    }
  }

  function runBenchmark(benchmark: Benchmark) {
    question = benchmark.question;
    // Auto-select all agents if none selected
    if (selectedAgents.size === 0 && agents.length > 0) {
      agents.forEach(a => selectedAgents.add(a.id));
      selectedAgents = selectedAgents;
    }
    // Scroll to top
    window.scrollTo({ top: 0, behavior: 'smooth' });
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

  let activeSession: any = null;

  async function viewSession(sessionId: string) {
    activeSessionId = sessionId;
    try {
      activeSession = await councilGetSession(sessionId);
      console.log("Session details:", activeSession);
    } catch (e: any) {
      error = `Failed to load session: ${e}`;
      activeSession = null;
    }
  }

  async function handleAddAgent() {
    if (!newAgentName.trim()) {
      error = "Agent name is required";
      return;
    }
    if (!newAgentModel.trim()) {
      error = "Model name is required";
      return;
    }

    loading = true;
    error = "";

    try {
      const agentId = await agentCreate(
        newAgentName,
        newAgentModel,
        newAgentPrompt || `You are ${newAgentName}, an AI assistant.`
      );
      console.log("Agent created:", agentId);
      
      // Reset form and reload
      newAgentName = "";
      newAgentModel = "qwen2.5-coder:7b";
      newAgentPrompt = "";
      showAddAgent = false;
      
      await loadAgents();
    } catch (e: any) {
      error = `Failed to create agent: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function handleDeleteAgent(agentId: string) {
    if (!confirm("Are you sure you want to delete this agent?")) {
      return;
    }

    loading = true;
    error = "";

    try {
      await agentDelete(agentId);
      await loadAgents();
      
      // Remove from selection if it was selected
      selectedAgents.delete(agentId);
      selectedAgents = selectedAgents;
    } catch (e: any) {
      error = `Failed to delete agent: ${e}`;
    } finally {
      loading = false;
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

  <!-- Agent Management -->
  <div class="agents-management">
    <div class="section-header">
      <h3>🤖 Available Agents</h3>
      <button 
        class="btn-add-agent" 
        on:click={() => showAddAgent = !showAddAgent}
        disabled={loading}
      >
        {showAddAgent ? "✖ Cancel" : "+ Add Agent"}
      </button>
    </div>

    {#if showAddAgent}
      <div class="add-agent-form">
        <div class="form-row">
          <div class="form-group">
            <label for="agent-name">Agent Name *</label>
            <input
              id="agent-name"
              type="text"
              bind:value={newAgentName}
              placeholder="e.g., Qwen Coder"
              disabled={loading}
            />
          </div>
          <div class="form-group">
            <label for="agent-model">Ollama Model *</label>
            <input
              id="agent-model"
              type="text"
              bind:value={newAgentModel}
              placeholder="qwen2.5-coder:7b"
              disabled={loading}
            />
            <span class="field-hint">Must be available on Ollama server</span>
          </div>
        </div>
        <div class="form-group">
          <label for="agent-prompt">System Prompt (optional)</label>
          <textarea
            id="agent-prompt"
            bind:value={newAgentPrompt}
            placeholder="e.g., You are an expert programmer specializing in Rust..."
            rows="3"
            disabled={loading}
          ></textarea>
        </div>
        <div class="form-actions">
          <button 
            class="btn-create" 
            on:click={handleAddAgent}
            disabled={loading || !newAgentName.trim() || !newAgentModel.trim()}
          >
            {loading ? "⏳ Creating..." : "✅ Create Agent"}
          </button>
        </div>
      </div>
    {/if}

    {#if agents.length === 0}
      <p class="empty-state">No agents yet. Create one above to get started.</p>
    {:else}
      <div class="agents-list">
        {#each agents as agent (agent.id)}
          <div class="agent-item">
            <div class="agent-info">
              <div class="agent-header">
                <span class="agent-name">{agent.name}</span>
                <button 
                  class="btn-delete-agent" 
                  on:click={() => handleDeleteAgent(agent.id)}
                  disabled={loading}
                  title="Delete agent"
                >
                  🗑️
                </button>
              </div>
              <div class="agent-model">Model: {agent.model_name}</div>
              {#if agent.system_prompt}
                <div class="agent-prompt">{agent.system_prompt}</div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

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

  <!-- History Tabs -->
  <div class="history-section">
    <div class="tabs">
      <button 
        class="tab-btn" 
        class:active={activeTab === "sessions"} 
        on:click={() => activeTab = "sessions"}
      >
        📜 Active Sessions
      </button>
      <button 
        class="tab-btn" 
        class:active={activeTab === "verdicts"} 
        on:click={() => activeTab = "verdicts"}
      >
        ⚖️ Verdict History
      </button>
      <button 
        class="tab-btn" 
        class:active={activeTab === "benchmarks"} 
        on:click={() => activeTab = "benchmarks"}
      >
        🧪 Benchmarks
      </button>
    </div>

    {#if activeTab === "sessions"}
      <div class="sessions-list">
        {#if sessions.length === 0}
          <p class="empty-state">No active sessions. Start a deliberation above.</p>
        {:else}
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
        {/if}
      </div>
    {:else if activeTab === "verdicts"}
      <div class="verdicts-list">
        {#if verdicts.length === 0}
          <p class="empty-state">No verdicts recorded yet.</p>
        {:else}
          {#each verdicts as verdict (verdict.id)}
            <div class="verdict-card">
              <div class="verdict-header">
                <span class="verdict-id">#{verdict.id.slice(0, 8)}</span>
                <span class="verdict-time">{new Date(verdict.created_at).toLocaleString()}</span>
                <span class="verdict-confidence">Confidence: {(verdict.confidence * 100).toFixed(0)}%</span>
              </div>
              <div class="verdict-question">{verdict.question}</div>
              <div class="verdict-result">
                <strong>Verdict:</strong> {verdict.verdict}
              </div>
              <div class="verdict-reasoning">
                {verdict.reasoning}
              </div>
              {#if verdict.dissent}
                <div class="verdict-dissent">
                  <strong>Dissent:</strong> {verdict.dissent}
                </div>
              {/if}
            </div>
          {/each}
        {/if}
      </div>
    {:else}
      <div class="benchmarks-list">
        {#each benchmarks as benchmark (benchmark.id)}
          <div class="benchmark-card">
            <div class="benchmark-header">
              <span class="benchmark-category">{benchmark.category}</span>
              <span class="benchmark-difficulty" class:hard={benchmark.difficulty === "Hard"}>{benchmark.difficulty}</span>
            </div>
            <div class="benchmark-question">{benchmark.question}</div>
            <div class="benchmark-trap">
              <strong>⚠️ Trap:</strong> {benchmark.trap_explanation}
            </div>
            <button class="run-benchmark-btn" on:click={() => runBenchmark(benchmark)}>
              🚀 Run This Test
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Active Session Details -->
  {#if activeSession}
    <div class="session-details">
      <h3>🔍 Session Details: #{activeSession.session_id.slice(0, 8)}</h3>
      
      <div class="detail-section">
        <h4>❓ Question</h4>
        <p class="question-text">{activeSession.question}</p>
      </div>

      <div class="detail-section">
        {#if activeSession.responses && activeSession.responses.length > 0}
          {@const uniqueModels = [...new Set(activeSession.responses.map((r: any) => r.model_name))].filter(Boolean)}
          <h4>👥 Participants ({uniqueModels.length})</h4>
          <div class="participants-list">
            {#each uniqueModels as modelName}
              <div class="participant-card">
                <div class="participant-icon">🤖</div>
                <div class="participant-info">
                  <div class="participant-name">{modelName}</div>
                  <div class="participant-model">
                    {activeSession.responses.filter((r: any) => r.model_name === modelName).length} responses
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <h4>👥 Participants (0)</h4>
          <p class="empty-state">No participants yet - session pending</p>
        {/if}
      </div>

      <div class="detail-section">
        <h4>💬 Responses ({activeSession.responses?.length || 0})</h4>
        {#if activeSession.responses && activeSession.responses.length > 0}
          <div class="responses-list">
            {#each activeSession.responses as response}
              <div class="response-card">
                <div class="response-header">
                  <strong>{response.model_name || response.agent_id || 'Unknown'}</strong>
                  <span class="response-timestamp">{new Date(response.timestamp * 1000).toLocaleTimeString()}</span>
                </div>
                <div class="response-text">{response.response}</div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="empty-state">No responses yet</p>
        {/if}
      </div>

      {#if activeSession.consensus_reached}
        <div class="detail-section consensus-section">
          <h4>✅ Consensus Reached</h4>
          <p class="consensus-text">{activeSession.final_answer || 'No final answer recorded'}</p>
        </div>
      {/if}
    </div>
  {/if}
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

  /* Agent Management */
  .agents-management {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 2rem;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .section-header h3 {
    margin: 0;
    color: #00d4ff;
    font-size: 1.2rem;
  }

  .btn-add-agent {
    padding: 0.5rem 1rem;
    background: #4caf50;
    border: none;
    border-radius: 6px;
    color: white;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-add-agent:hover:not(:disabled) {
    background: #45a049;
  }

  .btn-add-agent:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .add-agent-form {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(0, 212, 255, 0.2);
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 1rem;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .form-group {
    display: flex;
    flex-direction: column;
  }

  .form-group label {
    margin-bottom: 0.5rem;
    color: #aaa;
    font-size: 0.9rem;
    font-weight: 500;
  }

  .form-group input,
  .form-group textarea {
    padding: 0.75rem;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #e0e0e0;
    font-family: inherit;
  }

  .form-group input:focus,
  .form-group textarea:focus {
    outline: none;
    border-color: #00d4ff;
  }

  .form-group input:disabled,
  .form-group textarea:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .field-hint {
    margin-top: 0.25rem;
    font-size: 0.75rem;
    color: #666;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
  }

  .btn-create {
    padding: 0.75rem 1.5rem;
    background: linear-gradient(135deg, #00d4ff 0%, #0088cc 100%);
    border: none;
    border-radius: 6px;
    color: white;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-create:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 212, 255, 0.4);
  }

  .btn-create:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .agents-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .agent-item {
    background: rgba(0, 212, 255, 0.05);
    border: 1px solid rgba(0, 212, 255, 0.2);
    border-radius: 8px;
    padding: 1rem;
  }

  .agent-info {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .agent-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .agent-name {
    font-weight: 600;
    color: #00d4ff;
    font-size: 1rem;
  }

  .btn-delete-agent {
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 1.2rem;
    padding: 0.25rem 0.5rem;
    opacity: 0.6;
    transition: all 0.2s;
  }

  .btn-delete-agent:hover:not(:disabled) {
    opacity: 1;
    transform: scale(1.2);
  }

  .btn-delete-agent:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .agent-model {
    font-size: 0.85rem;
    color: #888;
    font-family: 'Courier New', monospace;
  }

  .agent-prompt {
    font-size: 0.85rem;
    color: #aaa;
    line-height: 1.4;
    padding: 0.5rem;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 4px;
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

  .agents-section {
    margin-bottom: 2rem;
  }

  .agents-section h3 {
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

  /* Session Details */
  .session-details {
    margin-top: 2rem;
    padding: 1.5rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
  }

  .detail-section {
    margin-top: 1.5rem;
  }

  .detail-section:first-child {
    margin-top: 0;
  }

  .detail-section h4 {
    margin: 0 0 1rem 0;
    color: #00d4ff;
    font-size: 1rem;
  }

  .question-text {
    padding: 1rem;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 8px;
    color: #e0e0e0;
    line-height: 1.5;
  }

  .participants-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.75rem;
  }

  .participant-card {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: rgba(0, 212, 255, 0.05);
    border: 1px solid rgba(0, 212, 255, 0.2);
    border-radius: 8px;
  }

  .participant-icon {
    font-size: 1.5rem;
  }

  .participant-info {
    flex: 1;
    min-width: 0;
  }

  .participant-name {
    font-weight: 500;
    color: #e0e0e0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .participant-model {
    font-size: 0.75rem;
    color: #888;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .responses-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .response-card {
    padding: 1rem;
    background: rgba(0, 0, 0, 0.2);
    border-left: 3px solid #00d4ff;
    border-radius: 4px;
  }

  .response-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
    color: #00d4ff;
    font-size: 0.9rem;
  }

  .response-timestamp {
    font-size: 0.8rem;
    color: #888;
  }

  .response-text {
    color: #e0e0e0;
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .consensus-section {
    border: 2px solid #4caf50;
    background: rgba(76, 175, 80, 0.05);
  }

  .consensus-text {
    padding: 1rem;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 8px;
    color: #4caf50;
    line-height: 1.5;
    font-weight: 500;
  }

  /* Tabs */
  .history-section {
    margin-top: 2rem;
  }

  .tabs {
    display: flex;
    gap: 1rem;
    margin-bottom: 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    padding-bottom: 0.5rem;
  }

  .tab-btn {
    background: transparent;
    border: none;
    color: #888;
    font-size: 1rem;
    font-weight: 600;
    padding: 0.5rem 1rem;
    cursor: pointer;
    transition: all 0.2s;
    border-bottom: 2px solid transparent;
  }

  .tab-btn:hover {
    color: #e0e0e0;
  }

  .tab-btn.active {
    color: #00d4ff;
    border-bottom-color: #00d4ff;
  }

  /* Verdict Cards */
  .verdicts-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .verdict-card {
    background: #0f1419;
    border: 1px solid #333;
    border-left: 3px solid #4caf50;
    border-radius: 8px;
    padding: 1.25rem;
  }

  .verdict-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.75rem;
    font-size: 0.85rem;
    color: #888;
  }

  .verdict-id {
    font-family: monospace;
    color: #4caf50;
  }

  .verdict-confidence {
    color: #00d4ff;
  }

  .verdict-question {
    font-weight: 600;
    color: #e0e0e0;
    margin-bottom: 1rem;
    font-size: 1.1rem;
  }

  .verdict-result {
    margin-bottom: 0.75rem;
    color: #4caf50;
    font-size: 1.05rem;
  }

  .verdict-reasoning {
    color: #ccc;
    line-height: 1.5;
    margin-bottom: 0.75rem;
    font-size: 0.95rem;
  }

  .verdict-dissent {
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    color: #ff9800;
    font-size: 0.9rem;
    font-style: italic;
  }

  /* Benchmarks */
  .benchmarks-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
  }

  .benchmark-card {
    background: #0f1419;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
  }

  .benchmark-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.75rem;
    font-size: 0.85rem;
  }

  .benchmark-category {
    color: #00d4ff;
    font-weight: 600;
  }

  .benchmark-difficulty {
    color: #4caf50;
  }

  .benchmark-difficulty.hard {
    color: #ff4444;
  }

  .benchmark-question {
    color: #e0e0e0;
    font-weight: 500;
    margin-bottom: 1rem;
    line-height: 1.4;
    flex: 1;
  }

  .benchmark-trap {
    background: rgba(255, 152, 0, 0.1);
    border: 1px solid rgba(255, 152, 0, 0.3);
    border-radius: 4px;
    padding: 0.75rem;
    margin-bottom: 1rem;
    font-size: 0.85rem;
    color: #ffb74d;
  }

  .run-benchmark-btn {
    width: 100%;
    padding: 0.75rem;
    background: #333;
    border: 1px solid #444;
    border-radius: 6px;
    color: white;
    cursor: pointer;
    transition: all 0.2s;
  }

  .run-benchmark-btn:hover {
    background: #444;
    border-color: #00d4ff;
    color: #00d4ff;
  }
</style>

<script lang="ts">
  import { onMount } from "svelte";
  import { askCouncil, getConfig, setDebug, getMetrics, type AppConfig, type PerformanceMetrics } from "./api";

  let question = "";
  let response = "";
  let loading = false;
  let config: AppConfig | null = null;
  let debugEnabled = true;
  let metrics: PerformanceMetrics | null = null;

  onMount(async () => {
    try {
      config = await getConfig();
      debugEnabled = config.debug_enabled;
      console.log("🔧 Config loaded:", config);
      await loadMetrics();
    } catch (error) {
      console.error("❌ Failed to load config:", error);
    }
  });

  async function loadMetrics() {
    try {
      metrics = await getMetrics();
    } catch (error) {
      console.error("❌ Failed to load metrics:", error);
    }
  }

  async function handleAskCouncil() {
    if (!question.trim()) return;
    
    loading = true;
    response = "";
    
    try {
      const result = await askCouncil(question);
      response = result;
      await loadMetrics(); // Refresh metrics after request
    } catch (error) {
      response = `Error: ${error}`;
      console.error("❌ Ask failed:", error);
    } finally {
      loading = false;
    }
  }

  async function toggleDebug() {
    debugEnabled = !debugEnabled;
    try {
      await setDebug(debugEnabled);
      console.log("🔧 Debug mode:", debugEnabled ? "ON" : "OFF");
    } catch (error) {
      console.error("❌ Failed to toggle debug:", error);
      debugEnabled = !debugEnabled; // Revert on error
    }
  }
</script>

<main>
  <header>
    <div class="header-content">
      <div>
        <h1>Council Of Dicks</h1>
        <p class="subtitle">And awaaaay we go! 🚀</p>
      </div>
      {#if config}
        <div class="status-panel">
          <div class="status-item">
            <span class="status-label">Model:</span>
            <span class="status-value">{config.ollama_model}</span>
          </div>
          <div class="status-item">
            <span class="status-label">Debug:</span>
            <button class="debug-toggle" on:click={toggleDebug}>
              {debugEnabled ? "🐛 ON" : "⚫ OFF"}
            </button>
          </div>
        </div>
      {/if}
    </div>
  </header>

  <div class="council-container">
    <div class="input-section">
      <textarea
        bind:value={question}
        placeholder="Ask the council a question..."
        rows="4"
        disabled={loading}
        on:keydown={(e) => {
          if (e.key === "Enter" && e.ctrlKey) {
            handleAskCouncil();
          }
        }}
      />
      <button on:click={handleAskCouncil} disabled={loading || !question.trim()}>
        {loading ? "🤔 Deliberating..." : "💬 Ask Council"}
      </button>
      <p class="hint">Tip: Ctrl+Enter to submit</p>
    </div>

    {#if response}
      <div class="response-section">
        <h3>Council Response:</h3>
        <div class="response-text">{response}</div>
      </div>
    {/if}
  </div>

  {#if metrics}
    <div class="metrics-panel">
      <h3>📊 Performance Metrics</h3>
      <div class="metrics-grid">
        <div class="metric">
          <span class="metric-label">Total Requests:</span>
          <span class="metric-value">{metrics.total_requests}</span>
        </div>
        <div class="metric">
          <span class="metric-label">Success Rate:</span>
          <span class="metric-value success">
            {metrics.total_requests > 0 
              ? ((metrics.successful_requests / metrics.total_requests) * 100).toFixed(1)
              : 0}%
          </span>
        </div>
        <div class="metric">
          <span class="metric-label">Failed:</span>
          <span class="metric-value error">{metrics.failed_requests}</span>
        </div>
        <div class="metric">
          <span class="metric-label">Avg Response Time:</span>
          <span class="metric-value">{metrics.average_response_time_ms.toFixed(0)}ms</span>
        </div>
      </div>
    </div>
  {/if}

  <footer>
    <p>
      🔥 NR5 IS ALIVE! 
      {#if config}
        <span class="server-url">({config.ollama_url})</span>
      {/if}
    </p>
    <p class="version">v0.1.0-alpha - Foundation Phase</p>
  </footer>
</main>

<style>
  :global(body) {
    font-family: system-ui, -apple-system, sans-serif;
    margin: 0;
    padding: 20px;
    background: #1a1a1a;
    color: #e0e0e0;
  }

  main {
    max-width: 800px;
    margin: 0 auto;
  }

  header {
    margin-bottom: 24px;
  }

  .header-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 24px;
  }

  h1 {
    color: #00ff88;
    margin-bottom: 8px;
    margin-top: 0;
  }

  .subtitle {
    color: #888;
    font-style: italic;
    margin: 0;
  }

  .status-panel {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: #2a2a2a;
    padding: 12px 16px;
    border-radius: 8px;
    border-left: 3px solid #00ff88;
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
  }

  .status-label {
    color: #888;
  }

  .status-value {
    color: #00ff88;
    font-family: monospace;
  }

  .debug-toggle {
    padding: 4px 12px;
    background: #1a1a1a;
    border: 1px solid #444;
    border-radius: 4px;
    color: #00ff88;
    cursor: pointer;
    font-size: 12px;
    transition: all 0.2s;
  }

  .debug-toggle:hover {
    background: #333;
    border-color: #00ff88;
  }

  .council-container {
    background: #2a2a2a;
    border-radius: 12px;
    padding: 24px;
    margin: 24px 0;
  }

  .input-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  textarea {
    width: 100%;
    padding: 12px;
    background: #1a1a1a;
    border: 2px solid #00ff88;
    border-radius: 8px;
    color: #e0e0e0;
    font-size: 16px;
    resize: vertical;
    font-family: inherit;
  }

  textarea:focus {
    outline: none;
    border-color: #00ffaa;
    box-shadow: 0 0 0 3px rgba(0, 255, 136, 0.1);
  }

  textarea:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button {
    padding: 12px 24px;
    background: #00ff88;
    color: #000;
    border: none;
    border-radius: 8px;
    font-size: 16px;
    font-weight: bold;
    cursor: pointer;
    transition: all 0.2s;
  }

  button:hover:not(:disabled) {
    background: #00ffaa;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 255, 136, 0.3);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .response-section {
    margin-top: 24px;
    padding-top: 24px;
    border-top: 2px solid #444;
  }

  .response-section h3 {
    color: #00ff88;
    margin-top: 0;
  }

  .response-text {
    background: #1a1a1a;
    padding: 16px;
    border-radius: 8px;
    border-left: 4px solid #00ff88;
    white-space: pre-wrap;
    line-height: 1.6;
  }

  .hint {
    margin: 0;
    color: #666;
    font-size: 13px;
    text-align: right;
  }

  footer {
    text-align: center;
    margin-top: 32px;
    padding-top: 24px;
    border-top: 1px solid #333;
    color: #666;
    font-size: 14px;
  }

  footer p {
    margin: 4px 0;
  }

  .server-url {
    color: #00ff88;
    font-family: monospace;
  }

  .version {
    font-size: 12px;
    color: #555;
  }

  .metrics-panel {
    background: #1a1a1a;
    border: 2px solid #00ff88;
    border-radius: 12px;
    padding: 20px;
    margin: 24px auto;
    max-width: 800px;
  }

  .metrics-panel h3 {
    color: #00ff88;
    margin: 0 0 16px 0;
    font-size: 18px;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 16px;
  }

  .metric {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .metric-label {
    color: #888;
    font-size: 13px;
  }

  .metric-value {
    color: #e0e0e0;
    font-size: 24px;
    font-weight: bold;
  }

  .metric-value.success {
    color: #00ff88;
  }

  .metric-value.error {
    color: #ff4444;
  }
</style>

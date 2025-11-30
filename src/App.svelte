<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let question = "";
  let response = "";
  let loading = false;

  async function askCouncil() {
    if (!question.trim()) return;
    
    loading = true;
    response = "";
    
    try {
      response = await invoke("ask_ollama", { question });
    } catch (error) {
      response = `Error: ${error}`;
    } finally {
      loading = false;
    }
  }
</script>

<main>
  <h1>Council Of Dicks</h1>
  <p class="subtitle">And awaaaay we go! 🚀</p>

  <div class="council-container">
    <div class="input-section">
      <textarea
        bind:value={question}
        placeholder="Ask the council a question..."
        rows="4"
        disabled={loading}
      />
      <button on:click={askCouncil} disabled={loading || !question.trim()}>
        {loading ? "Deliberating..." : "Ask Council"}
      </button>
    </div>

    {#if response}
      <div class="response-section">
        <h3>Council Response:</h3>
        <div class="response-text">{response}</div>
      </div>
    {/if}
  </div>

  <footer>
    <p>🔥 NR5 IS ALIVE! (192.168.1.5:11434)</p>
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

  h1 {
    color: #00ff88;
    text-align: center;
    margin-bottom: 10px;
  }

  .subtitle {
    text-align: center;
    color: #888;
    font-style: italic;
    margin-top: 0;
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

  footer {
    text-align: center;
    margin-top: 32px;
    color: #666;
    font-size: 14px;
  }

  footer p {
    margin: 0;
  }
</style>

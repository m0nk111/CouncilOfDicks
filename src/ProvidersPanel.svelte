<script lang="ts">
  import { onMount } from "svelte";
  import {
    providerAdd,
    providerList,
    providerRemove,
    providerTestConnection,
    providerGenerateUsername,
    getConfig,
    saveConfig,
    isTauriEnvironment,
    type AppConfig,
    type ProviderConfig,
    type ProviderType,
    type ProviderHealth,
  } from "./api";

  let providers: ProviderConfig[] = [];
  let showAddForm = false;
  let testingProvider: string | null = null;
  let providerHealth: Record<string, ProviderHealth> = {};
  let isTauri = false;
  let currentConfig: AppConfig | null = null;
  
  // Global config
  let defaultGenerationProvider = "guardian";
  let defaultGenerationModel = "qwen2.5-coder:7b";
  let openAICompatibleBaseUrl = "";
  let guardianBaseUrl = "http://127.0.0.1:11434";
  let guardianModel = "qwen3-coder-30b-q4km-32k:latest";
  let guardianApiKey = "";
  let globalOllamaUrl = "http://192.168.1.5:11434";
  let globalOllamaModel = "qwen2.5-coder:7b";
  let globalOllamaUsername = "";
  let globalOllamaPassword = "";
  let globalDebugEnabled = false;

  // Form state
  let formType: ProviderType = "ollama";
  let formUsername = "";
  let formDisplayName = "";
  let formEnabled = true;
  let formPriority = 1;

  // Ollama specific
  let ollamaBaseUrl = "http://192.168.1.5:11434";
  let ollamaDefaultModel = "qwen2.5-coder:7b";
  let ollamaEmbeddingModel = "nomic-embed-text";
  let ollamaTimeout = 120;

  // OpenAI specific
  let openaiApiKey = "";
  let openaiBaseUrl = "";
  let openaiOrganization = "";
  let openaiDefaultModel = "gpt-4-turbo-preview";

  // Anthropic specific
  let anthropicApiKey = "";
  let anthropicDefaultModel = "claude-3-opus-20240229";
  let anthropicVersion = "2023-06-01";

  async function loadGlobalConfig() {
    try {
      const config = await getConfig();
      currentConfig = config;
      defaultGenerationProvider = config.default_generation_provider || "guardian";
      defaultGenerationModel = config.default_generation_model || config.guardian_model || config.ollama_model;
      openAICompatibleBaseUrl = config.openai_base_url || "";
      guardianBaseUrl = config.guardian_url || "http://127.0.0.1:11434";
      guardianModel = config.guardian_model || config.ollama_model;
      guardianApiKey = config.guardian_api_key || "";
      globalOllamaUrl = config.ollama_url;
      globalOllamaModel = config.ollama_model;
      globalOllamaUsername = config.ollama_username || "";
      globalOllamaPassword = config.ollama_password || "";
      globalDebugEnabled = config.debug_enabled;
    } catch (error) {
      console.error("Failed to load config:", error);
    }
  }

  async function handleSaveGlobalConfig() {
    try {
      if (!currentConfig) {
        throw new Error("Global config not loaded yet");
      }

      const newConfig = {
        ...currentConfig,
        default_generation_provider: defaultGenerationProvider,
        default_generation_model: defaultGenerationModel || null,
        openai_base_url: openAICompatibleBaseUrl || null,
        guardian_url: guardianBaseUrl || null,
        guardian_model: guardianModel || null,
        guardian_api_key: guardianApiKey || null,
        ollama_url: globalOllamaUrl,
        ollama_model: globalOllamaModel,
        ollama_username: globalOllamaUsername || null,
        ollama_password: globalOllamaPassword || null,
        debug_enabled: globalDebugEnabled,
      };
      await saveConfig(newConfig);
      alert("Configuration saved!");
      currentConfig = newConfig;
    } catch (error) {
      alert(`Failed to save config: ${error}`);
    }
  }

  async function loadProviders() {
    try {
      providers = await providerList();
    } catch (error) {
      console.error("Failed to load providers:", error);
    }
  }

  onMount(() => {
    isTauri = isTauriEnvironment();
    loadGlobalConfig();
    loadProviders();
  });

  async function handleAddProvider() {
    const id = `${formType}_${Date.now()}`;

    let config: any = {
      id,
      username: formUsername,
      display_name: formDisplayName,
      provider_type: formType,
      enabled: formEnabled,
      priority: formPriority,
    };

    if (formType === "ollama") {
      config.config = {
        type: "Ollama",
        base_url: ollamaBaseUrl,
        default_model: ollamaDefaultModel,
        embedding_model: ollamaEmbeddingModel,
        timeout_seconds: ollamaTimeout,
      };
    } else if (formType === "openai") {
      config.config = {
        type: "OpenAI",
        api_key: openaiApiKey,
        base_url: openaiBaseUrl || undefined,
        organization: openaiOrganization || undefined,
        default_model: openaiDefaultModel,
      };
    } else if (formType === "anthropic") {
      config.config = {
        type: "Anthropic",
        api_key: anthropicApiKey,
        default_model: anthropicDefaultModel,
        version: anthropicVersion,
      };
    }

    try {
      await providerAdd(config);
      await loadProviders();
      resetForm();
      showAddForm = false;
    } catch (error) {
      alert(`Failed to add provider: ${error}`);
    }
  }

  async function handleRemoveProvider(id: string) {
    if (!confirm("Are you sure you want to remove this provider?")) {
      return;
    }

    try {
      await providerRemove(id);
      await loadProviders();
    } catch (error) {
      alert(`Failed to remove provider: ${error}`);
    }
  }

  async function handleTestConnection(id: string) {
    testingProvider = id;
    try {
      const health = await providerTestConnection(id);
      providerHealth[id] = health;
    } catch (error) {
      providerHealth[id] = {
        healthy: false,
        error: String(error),
      };
    }
    testingProvider = null;
  }

  async function handleGenerateUsername() {
    try {
      const modelName =
        formType === "ollama"
          ? ollamaDefaultModel
          : formType === "openai"
            ? openaiDefaultModel
            : anthropicDefaultModel;

      const username = await providerGenerateUsername(modelName, formType);
      formUsername = username;
      formDisplayName = `${formType.toUpperCase()} ${modelName}`;
    } catch (error) {
      alert(`Failed to generate username: ${error}`);
    }
  }

  function resetForm() {
    formUsername = "";
    formDisplayName = "";
    formEnabled = true;
    formPriority = 1;
    ollamaBaseUrl = "http://192.168.1.5:11434";
    ollamaDefaultModel = "qwen2.5-coder:7b";
    ollamaEmbeddingModel = "nomic-embed-text";
    openaiApiKey = "";
    anthropicApiKey = "";
  }

  function getProviderIcon(type: ProviderType): string {
    switch (type) {
      case "ollama":
        return "🦙";
      case "openai":
        return "🤖";
      case "anthropic":
        return "🧠";
      case "localembeddings":
        return "💾";
      default:
        return "❓";
    }
  }

  // Load providers on mount
  loadProviders();
</script>

<div class="providers-panel">
  <div class="panel-header">
    <h2>🤖 AI Providers</h2>
    <button class="btn-add" on:click={() => (showAddForm = !showAddForm)}>
      {showAddForm ? "✖ Cancel" : "+ Add Provider"}
    </button>
  </div>

  <!-- Global Ollama Configuration -->
  <div class="global-config">
    <div class="config-header">
      <h3>🌐 Generation Routing Configuration</h3>
      {#if isTauri}
        <button class="btn-save" on:click={handleSaveGlobalConfig}>💾 Save Config</button>
      {/if}
    </div>
    <div class="config-grid">
      <div class="config-item">
        <label for="default-generation-provider">Default Provider</label>
        <select
          id="default-generation-provider"
          bind:value={defaultGenerationProvider}
          disabled={!isTauri}
        >
          <option value="guardian">Guardian (Local OpenAI-Compatible)</option>
          <option value="ollama">Ollama / Guardian</option>
          <option value="openai">OpenAI-Compatible</option>
          <option value="openrouter">OpenRouter</option>
          <option value="google">Google Gemini</option>
        </select>
        <span class="config-hint">Used by generic generation flows like question generation and fallback council routing.</span>
      </div>
      <div class="config-item">
        <label for="guardian-base-url">Guardian Base URL</label>
        <input
          id="guardian-base-url"
          type="text"
          bind:value={guardianBaseUrl}
          placeholder="http://127.0.0.1:11434"
          readonly={!isTauri}
        />
        <span class="config-hint">Local Guardian proxy. The app will automatically use its OpenAI-compatible /v1 API.</span>
      </div>
      <div class="config-item">
        <label for="guardian-model">Guardian Local Model</label>
        <input
          id="guardian-model"
          type="text"
          bind:value={guardianModel}
          placeholder="qwen3-coder-30b-q4km-32k:latest"
          readonly={!isTauri}
        />
        <span class="config-hint">Primary local model for Guardian-routed flows and local agents.</span>
      </div>
      <div class="config-item">
        <label for="guardian-api-key">Guardian API Key</label>
        <input
          id="guardian-api-key"
          type="password"
          bind:value={guardianApiKey}
          placeholder="Optional if auto-discovered locally"
          readonly={!isTauri}
        />
        <span class="config-hint">Bearer token for Guardian. If left empty, TCOD tries to auto-discover a local key for your handle.</span>
      </div>
      <div class="config-item">
        <label for="default-generation-model">Default Generation Model</label>
        <input
          id="default-generation-model"
          type="text"
          bind:value={defaultGenerationModel}
          placeholder="qwen3-coder-30b-q4km-32k:latest"
          readonly={!isTauri}
        />
        <span class="config-hint">Applies to provider-agnostic flows. Leave aligned with your primary backend.</span>
      </div>
      <div class="config-item">
        <label for="openai-compatible-base-url">OpenAI-Compatible Base URL</label>
        <input
          id="openai-compatible-base-url"
          type="text"
          bind:value={openAICompatibleBaseUrl}
          placeholder="https://your-endpoint.example/v1"
          readonly={!isTauri}
        />
        <span class="config-hint">Optional. Use this for RunPod or other remote OpenAI-compatible endpoints when provider = openai.</span>
      </div>
      <div class="config-item">
        <label for="global-ollama-url">Ollama Server URL</label>
        <input 
          id="global-ollama-url"
          type="text" 
          bind:value={globalOllamaUrl} 
          placeholder="http://192.168.1.5:11434"
          readonly={!isTauri}
        />
        <span class="config-hint">Guardian/Ollama compatibility endpoint for explicit Ollama flows and local backends.</span>
      </div>
      <div class="config-item">
        <label for="global-ollama-model">Ollama / Guardian Model</label>
        <input 
          id="global-ollama-model"
          type="text" 
          bind:value={globalOllamaModel} 
          placeholder="qwen2.5-coder:7b"
          readonly={!isTauri}
        />
        <span class="config-hint">Still used by explicit Ollama endpoints and backward-compatible local flows.</span>
      </div>
      <div class="config-item">
        <label for="global-ollama-username">Guardian / Ollama Username</label>
        <input 
          id="global-ollama-username"
          type="text" 
          bind:value={globalOllamaUsername} 
          placeholder="Optional basic auth username"
          readonly={!isTauri}
        />
      </div>
      <div class="config-item">
        <label for="global-ollama-password">Guardian / Ollama Password</label>
        <input 
          id="global-ollama-password"
          type="password" 
          bind:value={globalOllamaPassword} 
          placeholder="Optional basic auth password"
          readonly={!isTauri}
        />
      </div>
      <div class="config-item">
        <span class="config-label">Debug Mode</span>
        <div class="toggle-wrapper">
          <span>{globalDebugEnabled ? "Enabled" : "Disabled"}</span>
          <span class="config-hint">{isTauri ? "Can be changed via debug toggle" : "Cannot be changed from web UI"}</span>
        </div>
      </div>
    </div>
    {#if !isTauri}
      <p class="config-note">
        ℹ️ Global config is read-only in web mode. To change, edit config file or use Tauri app.
      </p>
    {/if}
  </div>

  {#if showAddForm}
    <div class="add-form">
      <h3>Add New Provider</h3>

      <div class="form-group">
        <label for="provider-type">Provider Type</label>
        <select id="provider-type" bind:value={formType}>
          <option value="ollama">Ollama (Local/Network)</option>
          <option value="openai">OpenAI</option>
          <option value="anthropic">Anthropic (Claude)</option>
        </select>
      </div>

      <div class="form-group">
        <label for="provider-username">Username</label>
        <div class="input-with-button">
          <input
            id="provider-username"
            type="text"
            bind:value={formUsername}
            placeholder="e.g., CodeWhisperer, OracleGPT"
          />
          <button class="btn-secondary" on:click={handleGenerateUsername}
            >✨ Generate</button
          >
        </div>
        <small>Unique identifier for this AI agent</small>
      </div>

      <div class="form-group">
        <label for="provider-display-name">Display Name</label>
        <input
          id="provider-display-name"
          type="text"
          bind:value={formDisplayName}
          placeholder="e.g., My Local Qwen, GPT-4 Production"
        />
      </div>

      {#if formType === "ollama"}
        <div class="form-group">
          <label for="ollama-base-url">Base URL</label>
          <input
            id="ollama-base-url"
            type="text"
            bind:value={ollamaBaseUrl}
            placeholder="http://192.168.1.5:11434"
          />
        </div>

        <div class="form-group">
          <label for="ollama-default-model">Default Model</label>
          <input
            id="ollama-default-model"
            type="text"
            bind:value={ollamaDefaultModel}
            placeholder="qwen2.5-coder:7b"
          />
        </div>

        <div class="form-group">
          <label for="ollama-embedding-model">Embedding Model</label>
          <input
            id="ollama-embedding-model"
            type="text"
            bind:value={ollamaEmbeddingModel}
            placeholder="nomic-embed-text"
          />
        </div>
      {/if}

      {#if formType === "openai"}
        <div class="form-group">
          <label for="openai-api-key">API Key</label>
          <input
            id="openai-api-key"
            type="password"
            bind:value={openaiApiKey}
            placeholder="sk-..."
          />
          <small>Your OpenAI API key</small>
        </div>

        <div class="form-group">
          <label for="openai-default-model">Default Model</label>
          <input
            id="openai-default-model"
            type="text"
            bind:value={openaiDefaultModel}
            placeholder="gpt-4-turbo-preview"
          />
        </div>

        <div class="form-group">
          <label for="openai-base-url">Base URL (optional)</label>
          <input
            id="openai-base-url"
            type="text"
            bind:value={openaiBaseUrl}
            placeholder="https://api.openai.com/v1"
          />
        </div>

        <div class="form-group">
          <label for="openai-organization">Organization (optional)</label>
          <input
            id="openai-organization"
            type="text"
            bind:value={openaiOrganization}
            placeholder="org-..."
          />
        </div>
      {/if}

      {#if formType === "anthropic"}
        <div class="form-group">
          <label for="anthropic-api-key">API Key</label>
          <input
            id="anthropic-api-key"
            type="password"
            bind:value={anthropicApiKey}
            placeholder="sk-ant-..."
          />
          <small>Your Anthropic API key</small>
        </div>

        <div class="form-group">
          <label for="anthropic-default-model">Default Model</label>
          <input
            id="anthropic-default-model"
            type="text"
            bind:value={anthropicDefaultModel}
            placeholder="claude-3-opus-20240229"
          />
        </div>
      {/if}

      <div class="form-row">
        <div class="form-group">
          <label>
            <input type="checkbox" bind:checked={formEnabled} />
            Enabled
          </label>
        </div>

        <div class="form-group">
          <label for="provider-priority">Priority</label>
          <input id="provider-priority" type="number" bind:value={formPriority} min="0" max="100" />
        </div>
      </div>

      <div class="form-actions">
        <button class="btn-cancel" on:click={() => (showAddForm = false)}
          >Cancel</button
        >
        <button class="btn-primary" on:click={handleAddProvider}
          >Add Provider</button
        >
      </div>
    </div>
  {/if}

  <div class="providers-list">
    {#if providers.length === 0}
      <div class="empty-state">
        <p>No providers configured yet</p>
        <p class="subtitle">
          Add your first AI provider to start using the council
        </p>
      </div>
    {:else}
      {#each providers as provider (provider.id)}
        <div class="provider-card" class:disabled={!provider.enabled}>
          <div class="provider-header">
            <span class="provider-icon">{getProviderIcon(provider.provider_type)}</span>
            <div class="provider-info">
              <h4>{provider.display_name}</h4>
              <small>@{provider.username}</small>
            </div>
            <div class="provider-status">
              {#if provider.enabled}
                <span class="badge badge-success">Active</span>
              {:else}
                <span class="badge badge-inactive">Disabled</span>
              {/if}
            </div>
          </div>

          <div class="provider-details">
            <div class="detail-row">
              <span class="label">Type:</span>
              <span class="value">{provider.provider_type}</span>
            </div>
            <div class="detail-row">
              <span class="label">Priority:</span>
              <span class="value">{provider.priority}</span>
            </div>

            {#if "base_url" in provider.config}
              <div class="detail-row">
                <span class="label">URL:</span>
                <span class="value">{provider.config.base_url}</span>
              </div>
            {/if}

            {#if "default_model" in provider.config}
              <div class="detail-row">
                <span class="label">Model:</span>
                <span class="value">{provider.config.default_model}</span>
              </div>
            {/if}
          </div>

          {#if providerHealth[provider.id]}
            <div class="health-status" class:healthy={providerHealth[provider.id].healthy}>
              {#if providerHealth[provider.id].healthy}
                ✅ Healthy
                {#if providerHealth[provider.id].latency_ms}
                  ({providerHealth[provider.id].latency_ms}ms)
                {/if}
              {:else}
                ❌ {providerHealth[provider.id].error || "Unhealthy"}
              {/if}
            </div>
          {/if}

          <div class="provider-actions">
            <button
              class="btn-test"
              on:click={() => handleTestConnection(provider.id)}
              disabled={testingProvider === provider.id}
            >
              {testingProvider === provider.id ? "⏳ Testing..." : "🔍 Test"}
            </button>
            <button
              class="btn-remove"
              on:click={() => handleRemoveProvider(provider.id)}
            >
              🗑️ Remove
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .providers-panel {
    background: #1e1e1e;
    border-radius: 12px;
    padding: 20px;
    margin: 20px 0;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .panel-header h2 {
    margin: 0;
    color: #fff;
  }

  .btn-add {
    background: #4caf50;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 600;
  }

  .btn-add:hover {
    background: #45a049;
  }

  .global-config {
    background: linear-gradient(135deg, rgba(0, 212, 255, 0.05) 0%, rgba(0, 136, 204, 0.05) 100%);
    border: 1px solid rgba(0, 212, 255, 0.2);
    padding: 1.5rem;
    border-radius: 12px;
    margin-bottom: 2rem;
  }

  .global-config h3 {
    margin: 0 0 1rem 0;
    color: #00d4ff;
    font-size: 1.1rem;
  }

  .config-grid {
    display: grid;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .config-item label {
    display: block;
    margin-bottom: 0.5rem;
    color: #aaa;
    font-size: 0.9rem;
    font-weight: 500;
  }

  .config-item input {
    width: 100%;
    padding: 0.75rem;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #e0e0e0;
    font-family: 'Courier New', monospace;
  }

  .config-item select {
    width: 100%;
    padding: 0.75rem;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #e0e0e0;
  }

  .config-item input:read-only,
  .config-item select:disabled {
    cursor: not-allowed;
    opacity: 0.7;
  }

  .config-hint {
    display: block;
    margin-top: 0.25rem;
    font-size: 0.75rem;
    color: #666;
  }

  .toggle-wrapper {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .toggle-wrapper span:first-child {
    color: #e0e0e0;
    font-weight: 500;
  }

  .config-note {
    margin: 0;
    padding: 0.75rem;
    background: rgba(0, 136, 204, 0.1);
    border-left: 3px solid #0088cc;
    border-radius: 4px;
    color: #aaa;
    font-size: 0.85rem;
  }

  .add-form {
    background: #2a2a2a;
    padding: 20px;
    border-radius: 8px;
    margin-bottom: 20px;
  }

  .add-form h3 {
    margin-top: 0;
    color: #fff;
  }

  .form-group {
    margin-bottom: 15px;
  }

  .form-group label {
    display: block;
    color: #ccc;
    margin-bottom: 5px;
    font-weight: 500;
  }

  .form-group input[type="text"],
  .form-group input[type="password"],
  .form-group input[type="number"],
  .form-group select {
    width: 100%;
    padding: 10px;
    background: #1e1e1e;
    border: 1px solid #444;
    border-radius: 4px;
    color: #fff;
    font-size: 14px;
  }

  .form-group small {
    display: block;
    color: #888;
    margin-top: 5px;
    font-size: 12px;
  }

  .input-with-button {
    display: flex;
    gap: 10px;
  }

  .input-with-button input {
    flex: 1;
  }

  .btn-secondary {
    background: #2196f3;
    color: white;
    border: none;
    padding: 10px 15px;
    border-radius: 4px;
    cursor: pointer;
    white-space: nowrap;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 15px;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 20px;
  }

  .btn-primary {
    background: #4caf50;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 600;
  }

  .btn-cancel {
    background: #666;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 6px;
    cursor: pointer;
  }

  .providers-list {
    display: grid;
    gap: 15px;
  }

  .empty-state {
    text-align: center;
    padding: 40px;
    color: #888;
  }

  .empty-state .subtitle {
    color: #666;
    font-size: 14px;
  }

  .provider-card {
    background: #2a2a2a;
    border-radius: 8px;
    padding: 15px;
    border: 2px solid transparent;
    transition: border-color 0.2s;
  }

  .provider-card:hover {
    border-color: #4caf50;
  }

  .provider-card.disabled {
    opacity: 0.6;
  }

  .provider-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 10px;
  }

  .provider-icon {
    font-size: 32px;
  }

  .provider-info {
    flex: 1;
  }

  .provider-info h4 {
    margin: 0;
    color: #fff;
  }

  .provider-info small {
    color: #888;
  }

  .badge {
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
  }

  .badge-success {
    background: #4caf50;
    color: white;
  }

  .badge-inactive {
    background: #666;
    color: white;
  }

  .provider-details {
    padding: 10px 0;
    border-top: 1px solid #444;
    border-bottom: 1px solid #444;
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    padding: 4px 0;
    font-size: 14px;
  }

  .detail-row .label {
    color: #888;
  }

  .detail-row .value {
    color: #fff;
    font-family: monospace;
  }

  .health-status {
    margin-top: 10px;
    padding: 8px;
    border-radius: 4px;
    font-size: 14px;
  }

  .health-status.healthy {
    background: #1b5e20;
    color: #a5d6a7;
  }

  .health-status:not(.healthy) {
    background: #b71c1c;
    color: #ef9a9a;
  }

  .provider-actions {
    display: flex;
    gap: 10px;
  }

  .config-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .btn-save {
    background: #4caf50;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 600;
  }

  .btn-save:hover {
    background: #45a049;
  }
</style>

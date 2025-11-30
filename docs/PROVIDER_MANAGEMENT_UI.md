# Provider Management UI - User Guide

## Overview

De Provider Management UI laat je **meerdere AI providers configureren** via een gebruiksvriendelijke interface. Je kunt Ollama servers toevoegen, commerciële API keys invoeren, en elke AI agent een unieke username geven.

---

## UI Layout

```
╔════════════════════════════════════════════════════════════╗
║  🤖 AI Providers                    [+ Add Provider]      ║
╠════════════════════════════════════════════════════════════╣
║                                                            ║
║  ┌────────────────────────────────────────────────────┐  ║
║  │  Add New Provider                                   │  ║
║  │  ┌──────────────────────────────────────────────┐  │  ║
║  │  │ Provider Type: [Ollama ▼]                    │  │  ║
║  │  ├──────────────────────────────────────────────┤  │  ║
║  │  │ Username: [qwen_coder_7b]  [✨ Generate]    │  │  ║
║  │  │ (Unique identifier for this AI agent)        │  │  ║
║  │  ├──────────────────────────────────────────────┤  │  ║
║  │  │ Display Name: [My Local Qwen]               │  │  ║
║  │  ├──────────────────────────────────────────────┤  │  ║
║  │  │ Base URL: [http://192.168.1.5:11434]        │  │  ║
║  │  │ Default Model: [qwen2.5-coder:7b]           │  │  ║
║  │  │ Embedding Model: [nomic-embed-text]         │  │  ║
║  │  ├──────────────────────────────────────────────┤  │  ║
║  │  │ ☑ Enabled    Priority: [1]                  │  │  ║
║  │  └──────────────────────────────────────────────┘  │  ║
║  │                        [Cancel] [Add Provider]      │  ║
║  └────────────────────────────────────────────────────┘  ║
║                                                            ║
║  ┌────────────────────────────────────────────────────┐  ║
║  │  🦙  My Local Qwen                   [Active]      │  ║
║  │      @qwen_coder_7b                                │  ║
║  │  ─────────────────────────────────────────────     │  ║
║  │  Type:     ollama                                  │  ║
║  │  Priority: 1                                       │  ║
║  │  URL:      http://192.168.1.5:11434                │  ║
║  │  Model:    qwen2.5-coder:7b                        │  ║
║  │  ─────────────────────────────────────────────     │  ║
║  │  ✅ Healthy (243ms)                                 │  ║
║  │  [🔍 Test]                    [🗑️ Remove]          │  ║
║  └────────────────────────────────────────────────────┘  ║
║                                                            ║
║  ┌────────────────────────────────────────────────────┐  ║
║  │  🤖  OpenAI GPT-4                   [Active]       │  ║
║  │      @gpt4_production                              │  ║
║  │  ─────────────────────────────────────────────     │  ║
║  │  Type:     openai                                  │  ║
║  │  Priority: 2                                       │  ║
║  │  Model:    gpt-4-turbo-preview                     │  ║
║  │  [🔍 Test]                    [🗑️ Remove]          │  ║
║  └────────────────────────────────────────────────────┘  ║
║                                                            ║
║  ┌────────────────────────────────────────────────────┐  ║
║  │  🧠  Claude Opus                   [Disabled]      │  ║
║  │      @claude_opus_sage                             │  ║
║  │  ─────────────────────────────────────────────     │  ║
║  │  Type:     anthropic                               │  ║
║  │  Priority: 3                                       │  ║
║  │  Model:    claude-3-opus-20240229                  │  ║
║  │  [🔍 Test]                    [🗑️ Remove]          │  ║
║  └────────────────────────────────────────────────────┘  ║
╚════════════════════════════════════════════════════════════╝
```

---

## Supported Providers

### 1. **Ollama** (Local/Network)
**Use Case:** Zelf-gehoste models, lokaal netwerk  
**Config:**
- Base URL: `http://192.168.1.5:11434`
- Default Model: `qwen2.5-coder:7b`
- Embedding Model: `nomic-embed-text`
- Timeout: `120` seconden

**Voorbeeld Username:** `qwen_coder_7b`, `llama3_assistant`, `mistral_local`

---

### 2. **OpenAI**
**Use Case:** GPT-4, GPT-3.5, commerciële API  
**Config:**
- API Key: `sk-proj-...` (required)
- Base URL: `https://api.openai.com/v1` (optional)
- Organization: `org-...` (optional)
- Default Model: `gpt-4-turbo-preview`

**Voorbeeld Username:** `gpt4_production`, `gpt35_rapid`, `openai_oracle`

---

### 3. **Anthropic** (Claude)
**Use Case:** Claude 3 Opus/Sonnet/Haiku  
**Config:**
- API Key: `sk-ant-...` (required)
- Default Model: `claude-3-opus-20240229`
- Version: `2023-06-01`

**Voorbeeld Username:** `claude_opus_sage`, `claude_sonnet_quick`, `anthropic_ethicist`

---

## Features

### ✨ Username Generator
Klik op **"✨ Generate"** om automatisch een username te genereren op basis van:
- Provider type (ollama, openai, anthropic)
- Model name (qwen2.5-coder:7b → qwen_coder_7b)

**Toekomstige feature:** LLM genereert creatieve usernames op basis van model capabilities:
- `qwen2.5-coder:7b` → `"CodeWhisperer"` of `"TheArchitect"`
- `gpt-4` → `"OracleOfKnowledge"` of `"QuantumThink"`
- `claude-3-opus` → `"ThePhilosopher"` of `"EthicalGuardian"`

---

### 🔍 Test Connection
Elke provider kan getest worden via **"🔍 Test"** button:

**Succesvolle test:**
```
✅ Healthy (243ms)
```

**Gefaalde test:**
```
❌ Connection timeout: could not reach http://192.168.1.5:11434
```

---

### 🗑️ Remove Provider
Verwijder providers die je niet meer gebruikt. 

**Bevestiging:**
```
Are you sure you want to remove this provider?
[Cancel] [OK]
```

---

## Priority System

**Priority bepaalt fallback volgorde:**
- `0` = Hoogste prioriteit (eerst geprobeerd)
- `1-99` = Lagere prioriteit
- `100` = Laagste prioriteit (laatste poging)

**Voorbeeld scenario:**
```
1. local_embeddings (priority: 0) → Altijd eerst voor embeddings
2. ollama_local (priority: 1)     → Primaire voor text generation
3. openai_gpt4 (priority: 2)      → Fallback als Ollama down is
4. claude_opus (priority: 3)      → Laatste resort
```

---

## Config File Format

Providers worden opgeslagen in `providers.json`:

```json
{
  "version": "1.0",
  "providers": [
    {
      "id": "ollama_1701234567890",
      "username": "qwen_coder_7b",
      "display_name": "My Local Qwen",
      "provider_type": "ollama",
      "enabled": true,
      "priority": 1,
      "config": {
        "type": "Ollama",
        "base_url": "http://192.168.1.5:11434",
        "default_model": "qwen2.5-coder:7b",
        "embedding_model": "nomic-embed-text",
        "timeout_seconds": 120
      }
    },
    {
      "id": "openai_1701234567891",
      "username": "gpt4_production",
      "display_name": "OpenAI GPT-4",
      "provider_type": "openai",
      "enabled": true,
      "priority": 2,
      "config": {
        "type": "OpenAI",
        "api_key": "sk-proj-...",
        "base_url": null,
        "organization": null,
        "default_model": "gpt-4-turbo-preview"
      }
    }
  ],
  "default_generation_provider": "ollama_1701234567890",
  "default_embedding_provider": "ollama_1701234567890"
}
```

---

## Security

### API Key Storage
- ❌ **NIET** in git committen
- ✅ Lokaal in `providers.json` (in `.gitignore`)
- 🔒 TODO: Encrypt met OS keyring

### Validatie
- OpenAI keys moeten starten met `sk-`
- Anthropic keys moeten starten met `sk-ant-`
- URLs moeten `http://` of `https://` zijn

---

## Usage in Code

### Frontend (TypeScript)
```typescript
import { providerAdd, providerList, providerTestConnection } from "./api";

// Lijst alle providers
const providers = await providerList();

// Voeg nieuwe provider toe
await providerAdd({
  id: "ollama_local",
  username: "qwen_coder",
  display_name: "Local Qwen",
  provider_type: "ollama",
  enabled: true,
  priority: 1,
  config: {
    type: "Ollama",
    base_url: "http://localhost:11434",
    default_model: "qwen2.5-coder:7b",
    embedding_model: "nomic-embed-text",
    timeout_seconds: 120,
  },
});

// Test connectie
const health = await providerTestConnection("ollama_local");
console.log(health.healthy ? "✅ Online" : "❌ Offline");
```

### Backend (Rust)
```rust
// Load config
let config = ProvidersConfig::load("providers.json")?;

// Get provider
let provider = config.get_provider("ollama_local").unwrap();

// Create provider instance
let ollama = OllamaProvider::new(
    provider.config.base_url.clone(),
    provider.config.default_model.clone(),
    logger,
);

// Test health
let health = ollama.health_check().await?;
```

---

## Keyboard Shortcuts

- `Ctrl + A` → Add new provider
- `Esc` → Cancel add form
- `Enter` → Submit form (when focused)

---

## Tips

1. **Meerdere Ollama servers:**  
   Je kunt meerdere Ollama instances toevoegen (e.g., `http://desktop:11434`, `http://server:11434`)

2. **Username betekenis:**  
   Kies descriptieve usernames zoals `qwen_coder_fast` vs `qwen_coder_accurate` voor verschillende configs

3. **Priority strategie:**  
   - Lokale models = low priority number (snel, goedkoop)
   - Cloud APIs = high priority number (slow, duur, maar beter)

4. **Test regelmatig:**  
   Gebruik "Test" button om te verifiëren dat providers nog werken

---

## Next Features

- [ ] Edit existing providers (nu alleen add/remove)
- [ ] Drag-and-drop priority reordering
- [ ] Import/export provider configs
- [ ] LLM-generated creative usernames
- [ ] Usage statistics per provider
- [ ] Cost tracking for paid APIs

---

**Klaar om te gebruiken!** 🚀

Open de TCOD app en navigeer naar de **🤖 AI Providers** panel om je eerste provider toe te voegen.

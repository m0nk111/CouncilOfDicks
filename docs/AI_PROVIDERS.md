# AI Provider Architecture

## Problem Statement

**Current Issues:**
1. ❌ **Hardcoded Ollama dependency** - Not truly standalone/portable
2. ❌ **Embeddings require network** - Knowledge Bank depends on external Ollama API
3. ❌ **Single provider** - Cannot mix OpenAI, Anthropic, local models
4. ❌ **No fallback** - If Ollama is down, entire system fails

**Requirements:**
- ✅ **Standalone operation** - Must work without network (embedded models)
- ✅ **Provider agnostic** - Support multiple AI backends
- ✅ **Hot-swappable** - Change providers without restarting
- ✅ **Mixed councils** - Different models from different providers in same deliberation
- ✅ **Portable** - Single binary with embedded capabilities
- ✅ **Configurable** - Easy to add new providers via config/plugins

---

## Architecture Design

### 1. Provider Trait

```rust
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Provider identification
    fn name(&self) -> &str;
    fn provider_type(&self) -> ProviderType;
    
    /// Core capabilities
    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse, ProviderError>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ProviderError>;
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError>;
    
    /// Health & status
    async fn health_check(&self) -> Result<ProviderHealth, ProviderError>;
    fn is_available(&self) -> bool;
    
    /// Configuration
    fn supports_embeddings(&self) -> bool;
    fn supports_streaming(&self) -> bool;
    fn max_context_length(&self) -> usize;
}

pub enum ProviderType {
    Network { requires_internet: bool },
    Local { bundled: bool },
    Hybrid,
}

pub struct GenerationRequest {
    pub model: String,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub temperature: f32,
    pub max_tokens: Option<usize>,
    pub stream: bool,
}

pub struct GenerationResponse {
    pub text: String,
    pub model: String,
    pub tokens_used: usize,
    pub finish_reason: FinishReason,
}

pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_length: usize,
    pub supports_embeddings: bool,
    pub supports_function_calling: bool,
}
```

### 2. Provider Implementations

#### A. Ollama Provider (Network)
```rust
pub struct OllamaProvider {
    base_url: String,
    timeout: Duration,
    logger: Arc<Logger>,
}

impl AIProvider for OllamaProvider {
    fn name(&self) -> &str { "Ollama" }
    fn provider_type(&self) -> ProviderType { 
        ProviderType::Network { requires_internet: false } 
    }
    
    async fn generate(&self, req: GenerationRequest) -> Result<GenerationResponse, ProviderError> {
        // Existing ask_ollama logic
    }
    
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ProviderError> {
        // Existing generate_embedding logic
    }
    
    fn supports_embeddings(&self) -> bool { true }
    fn max_context_length(&self) -> usize { 8192 } // Model-dependent
}
```

#### B. Local Embeddings Provider (Embedded)
```rust
pub struct LocalEmbeddingsProvider {
    model: EmbeddingModel, // rust-bert or candle-transformers
    logger: Arc<Logger>,
}

impl AIProvider for LocalEmbeddingsProvider {
    fn name(&self) -> &str { "Local Embeddings (all-MiniLM-L6-v2)" }
    fn provider_type(&self) -> ProviderType { 
        ProviderType::Local { bundled: true } 
    }
    
    async fn generate(&self, _req: GenerationRequest) -> Result<GenerationResponse, ProviderError> {
        Err(ProviderError::NotSupported("Text generation not supported".into()))
    }
    
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ProviderError> {
        // Run inference locally using rust-bert
        // Model bundled in binary (~80MB)
        let embedding = self.model.encode(&[text])?;
        Ok(embedding[0].clone())
    }
    
    fn supports_embeddings(&self) -> bool { true }
    fn is_available(&self) -> bool { true } // Always available
}
```

#### C. OpenAI Provider (Network + API Key)
```rust
pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
    organization: Option<String>,
    logger: Arc<Logger>,
}

impl AIProvider for OpenAIProvider {
    fn name(&self) -> &str { "OpenAI" }
    fn provider_type(&self) -> ProviderType { 
        ProviderType::Network { requires_internet: true } 
    }
    
    async fn generate(&self, req: GenerationRequest) -> Result<GenerationResponse, ProviderError> {
        // Call OpenAI API /v1/chat/completions
    }
    
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ProviderError> {
        // Call OpenAI API /v1/embeddings
    }
}
```

#### D. Anthropic Provider (Network + API Key)
```rust
pub struct AnthropicProvider {
    api_key: String,
    logger: Arc<Logger>,
}

impl AIProvider for AnthropicProvider {
    fn name(&self) -> &str { "Anthropic Claude" }
    fn provider_type(&self) -> ProviderType { 
        ProviderType::Network { requires_internet: true } 
    }
    
    async fn generate(&self, req: GenerationRequest) -> Result<GenerationResponse, ProviderError> {
        // Call Anthropic API /v1/messages
    }
    
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, ProviderError> {
        Err(ProviderError::NotSupported("Claude doesn't support embeddings".into()))
    }
}
```

### 3. Provider Registry

```rust
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn AIProvider>>,
    default_generation_provider: String,
    default_embedding_provider: String,
    logger: Arc<Logger>,
}

impl ProviderRegistry {
    pub fn new(logger: Arc<Logger>) -> Self {
        let mut registry = Self {
            providers: HashMap::new(),
            default_generation_provider: "local_embeddings".to_string(),
            default_embedding_provider: "local_embeddings".to_string(),
            logger,
        };
        
        // Register default providers
        registry.register_local_embeddings();
        
        registry
    }
    
    pub fn register(&mut self, id: String, provider: Arc<dyn AIProvider>) {
        self.logger.log(LogLevel::Info, "registry", 
            &format!("📦 Registered provider: {} ({})", provider.name(), id));
        self.providers.insert(id, provider);
    }
    
    pub fn get(&self, id: &str) -> Option<Arc<dyn AIProvider>> {
        self.providers.get(id).cloned()
    }
    
    pub fn get_generation_provider(&self) -> Arc<dyn AIProvider> {
        self.get(&self.default_generation_provider)
            .expect("Default generation provider must exist")
    }
    
    pub fn get_embedding_provider(&self) -> Arc<dyn AIProvider> {
        self.get(&self.default_embedding_provider)
            .expect("Default embedding provider must exist")
    }
    
    pub fn list_available(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
    
    fn register_local_embeddings(&mut self) {
        let provider = Arc::new(LocalEmbeddingsProvider::new(self.logger.clone()));
        self.register("local_embeddings".to_string(), provider);
    }
}
```

### 4. Configuration Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub provider_type: String, // "ollama", "openai", "anthropic", "local_embeddings"
    pub enabled: bool,
    pub priority: u32, // For fallback ordering
    pub config: ProviderSpecificConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderSpecificConfig {
    Ollama {
        base_url: String,
        default_model: String,
        timeout_seconds: u64,
    },
    OpenAI {
        api_key: String,
        base_url: Option<String>,
        organization: Option<String>,
        default_model: String,
    },
    Anthropic {
        api_key: String,
        default_model: String,
    },
    LocalEmbeddings {
        model_path: Option<String>, // If None, use bundled
    },
}
```

**Example config.json:**
```json
{
  "providers": [
    {
      "id": "local_embeddings",
      "provider_type": "local_embeddings",
      "enabled": true,
      "priority": 1,
      "config": {
        "type": "LocalEmbeddings",
        "model_path": null
      }
    },
    {
      "id": "ollama_local",
      "provider_type": "ollama",
      "enabled": true,
      "priority": 2,
      "config": {
        "type": "Ollama",
        "base_url": "http://192.168.1.5:11434",
        "default_model": "qwen2.5-coder:7b",
        "timeout_seconds": 120
      }
    },
    {
      "id": "openai_gpt4",
      "provider_type": "openai",
      "enabled": false,
      "priority": 3,
      "config": {
        "type": "OpenAI",
        "api_key": "sk-...",
        "base_url": null,
        "organization": null,
        "default_model": "gpt-4-turbo-preview"
      }
    }
  ],
  "default_generation_provider": "ollama_local",
  "default_embedding_provider": "local_embeddings"
}
```

---

## Implementation Plan

### Phase 1: Provider Trait & Local Embeddings
**Goal:** Standalone embeddings without network dependency

1. Create `src-tauri/src/providers/` module structure:
   ```
   providers/
   ├── mod.rs           # Trait definitions
   ├── registry.rs      # ProviderRegistry
   ├── local_embed.rs   # LocalEmbeddingsProvider
   ├── ollama.rs        # OllamaProvider (refactor existing)
   └── error.rs         # ProviderError
   ```

2. Add dependencies to `Cargo.toml`:
   ```toml
   rust-bert = "0.22"              # For local embeddings
   # OR
   candle-core = "0.6"             # Alternative: HF Candle
   candle-nn = "0.6"
   candle-transformers = "0.6"
   
   async-trait = "0.1"             # For trait async methods
   ```

3. Implement `LocalEmbeddingsProvider`:
   - Bundle all-MiniLM-L6-v2 model (~80MB)
   - Lazy initialization (load on first use)
   - Thread-safe inference (Arc<Mutex<Model>>)

4. Update `KnowledgeBank`:
   ```rust
   pub struct KnowledgeBank {
       pool: SqlitePool,
       logger: Arc<Logger>,
       embedding_provider: Arc<dyn AIProvider>, // Was: ollama_url + model
   }
   ```

5. Test:
   - Knowledge Bank works offline
   - Embeddings generated locally
   - Binary size increase acceptable (~80-100MB)

**Deliverable:** Standalone Knowledge Bank with local embeddings ✅

### Phase 2: Provider Registry & Ollama Refactor
**Goal:** Pluggable providers with backwards compatibility

1. Implement `ProviderRegistry` in `src-tauri/src/providers/registry.rs`
2. Refactor `OllamaProvider` to implement `AIProvider` trait
3. Update `AppState`:
   ```rust
   pub struct AppState {
       // ... existing fields
       pub provider_registry: Arc<Mutex<ProviderRegistry>>,
   }
   ```

4. Update `DeliberationEngine`:
   ```rust
   pub struct DeliberationEngine {
       logger: Arc<Logger>,
       provider_registry: Arc<Mutex<ProviderRegistry>>,
   }
   
   impl DeliberationEngine {
       async fn query_member(
           registry: Arc<Mutex<ProviderRegistry>>,
           member: CouncilMember,
           // ... other params
       ) -> Result<MemberResponse, String> {
           let registry = registry.lock().unwrap();
           let provider = registry.get(&member.provider_id)
               .ok_or("Provider not found")?;
           
           let response = provider.generate(GenerationRequest {
               model: member.model.clone(),
               prompt,
               system_prompt: Some(member.system_prompt.clone()),
               temperature: 0.7,
               max_tokens: None,
               stream: false,
           }).await?;
           
           // ... rest of logic
       }
   }
   ```

5. Update `CouncilMember`:
   ```rust
   pub struct CouncilMember {
       pub name: String,
       pub model: String,
       pub personality: String,
       pub system_prompt: String,
       pub provider_id: String, // NEW: "ollama_local", "openai_gpt4", etc.
   }
   ```

**Deliverable:** Multi-provider deliberations ✅

### Phase 3: OpenAI & Anthropic Providers
**Goal:** Cloud provider integration

1. Implement `OpenAIProvider` in `src-tauri/src/providers/openai.rs`
2. Implement `AnthropicProvider` in `src-tauri/src/providers/anthropic.rs`
3. Add Tauri commands:
   ```rust
   #[tauri::command]
   async fn provider_add(
       config: ProviderConfig,
       state: tauri::State<'_, AppState>,
   ) -> Result<String, String>
   
   #[tauri::command]
   fn provider_list(
       state: tauri::State<'_, AppState>,
   ) -> Result<Vec<String>, String>
   
   #[tauri::command]
   async fn provider_health_check(
       provider_id: String,
       state: tauri::State<'_, AppState>,
   ) -> Result<ProviderHealth, String>
   ```

4. Update UI to show provider status:
   ```svelte
   <div class="providers-panel">
     {#each providers as provider}
       <ProviderCard {provider} />
     {/each}
   </div>
   ```

**Deliverable:** Cloud provider support ✅

### Phase 4: Fallback & Resilience
**Goal:** Graceful degradation

1. Implement fallback chain:
   ```rust
   pub async fn generate_with_fallback(
       &self,
       request: GenerationRequest,
   ) -> Result<GenerationResponse, ProviderError> {
       let providers = self.get_providers_by_priority();
       
       for provider in providers {
           if !provider.is_available() {
               continue;
           }
           
           match provider.generate(request.clone()).await {
               Ok(response) => return Ok(response),
               Err(e) => {
                   self.logger.warn("registry", 
                       &format!("Provider {} failed: {}, trying next...", 
                       provider.name(), e));
                   continue;
               }
           }
       }
       
       Err(ProviderError::AllProvidersFailed)
   }
   ```

2. Add circuit breaker pattern:
   ```rust
   pub struct CircuitBreaker {
       failures: usize,
       last_failure: Option<Instant>,
       threshold: usize,
       timeout: Duration,
   }
   ```

**Deliverable:** Robust multi-provider system ✅

---

## Migration Strategy

### Backwards Compatibility

Existing code using `OllamaClient` continues to work:
```rust
// OLD CODE (still works)
let ollama = Arc::new(Mutex::new(OllamaClient::new(config, logger)));
let response = ollama.lock().unwrap().ask(&model, &prompt).await?;

// NEW CODE (recommended)
let registry = state.provider_registry.lock().unwrap();
let provider = registry.get("ollama_local").unwrap();
let response = provider.generate(request).await?;
```

### Config Migration

Auto-migrate old config to new format:
```rust
impl AppConfig {
    pub fn migrate_to_providers(&self) -> Vec<ProviderConfig> {
        vec![
            ProviderConfig {
                id: "ollama_local".to_string(),
                provider_type: "ollama".to_string(),
                enabled: true,
                priority: 1,
                config: ProviderSpecificConfig::Ollama {
                    base_url: self.ollama_url.clone(),
                    default_model: self.ollama_model.clone(),
                    timeout_seconds: 120,
                },
            },
            ProviderConfig {
                id: "local_embeddings".to_string(),
                provider_type: "local_embeddings".to_string(),
                enabled: true,
                priority: 0,
                config: ProviderSpecificConfig::LocalEmbeddings {
                    model_path: None,
                },
            },
        ]
    }
}
```

---

## Local Embeddings Technical Details

### Model Choice: all-MiniLM-L6-v2

**Why this model:**
- ✅ Small size: ~80MB (acceptable for bundling)
- ✅ Fast inference: ~50ms per embedding on CPU
- ✅ Good quality: 384-dimensional embeddings
- ✅ Widely used: Proven in production
- ✅ MIT licensed: No legal issues

**Implementation using rust-bert:**
```rust
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};

pub struct LocalEmbeddingsProvider {
    model: Arc<Mutex<SentenceEmbeddingsModel>>,
    logger: Arc<Logger>,
}

impl LocalEmbeddingsProvider {
    pub fn new(logger: Arc<Logger>) -> Result<Self, ProviderError> {
        logger.log(LogLevel::Info, "embeddings", 
            "🧠 Loading local embedding model (all-MiniLM-L6-v2)...");
        
        let model = SentenceEmbeddingsBuilder::remote(
            SentenceEmbeddingsModelType::AllMiniLmL6V2
        )
        .create_model()?;
        
        logger.log(LogLevel::Success, "embeddings", 
            "✅ Local embeddings ready (384-dim, ~50ms/embedding)");
        
        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            logger,
        })
    }
    
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, ProviderError> {
        let model = self.model.lock().unwrap();
        let embeddings = model.encode(&[text])?;
        Ok(embeddings[0].clone())
    }
}
```

**Bundling Strategy:**
1. Model downloaded on first build (build script)
2. Embedded in binary using `include_bytes!()` or cargo build resources
3. Lazy initialization (only loaded when needed)
4. Cached in memory after first use

**Alternative: Candle (HuggingFace)**
```rust
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};

// Similar approach but more control over model loading
```

---

## Benefits Summary

**Before (v0.4.0):**
- ❌ Network required for embeddings (Ollama dependency)
- ❌ Single provider (Ollama only)
- ❌ Not portable (requires external service)
- ❌ No fallback if Ollama is down

**After (v0.5.0):**
- ✅ **Fully standalone** - Local embeddings bundled in binary
- ✅ **Multi-provider** - Mix Ollama, OpenAI, Claude, local models
- ✅ **Portable** - Single binary, no external dependencies required
- ✅ **Resilient** - Automatic fallback between providers
- ✅ **Flexible** - Add providers via config without code changes
- ✅ **Future-proof** - Easy to add new AI services

---

## Performance Considerations

### Binary Size Impact
- Base app: ~20MB
- + Local embeddings: ~80MB
- **Total: ~100MB** (acceptable for standalone app)

### Memory Usage
- Embedding model: ~200MB RAM when loaded
- Lazy loading: Only loaded when Knowledge Bank used
- Model caching: Persistent in memory after first use

### Inference Speed
- Local embeddings: ~50ms per text (CPU)
- Ollama embeddings: ~200ms (network + GPU)
- OpenAI embeddings: ~500ms (API call)

**Recommendation:** Use local embeddings by default, offer cloud options for better quality if desired.

---

## Security Considerations

1. **API Key Storage:**
   - Encrypted storage using system keyring (keytar/keyring-rs)
   - Never log API keys
   - Clear memory after use

2. **Provider Validation:**
   - Health checks before use
   - Rate limiting to prevent abuse
   - Timeout enforcement

3. **Local Model Safety:**
   - Model integrity checks (SHA256)
   - Sandboxed execution
   - Resource limits (CPU, memory)

---

## Next Steps

1. ✅ Create this design document
2. ⏳ Implement Phase 1: Local embeddings + Provider trait
3. ⏳ Implement Phase 2: Provider registry + Ollama refactor
4. ⏳ Implement Phase 3: Cloud providers (OpenAI, Anthropic)
5. ⏳ Implement Phase 4: Fallback & resilience
6. ⏳ Update documentation & examples
7. ⏳ Test standalone operation on clean system

**Target Version:** v0.5.0-alpha  
**Estimated Effort:** 2-3 weeks (phased implementation)  
**Priority:** HIGH (critical for standalone/portable promise)

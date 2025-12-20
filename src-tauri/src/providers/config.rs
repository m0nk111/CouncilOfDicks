use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Available roles an AI agent can choose
pub const AVAILABLE_ROLES: &[&str] = &[
    "Skeptic",      // Questions assumptions, demands evidence
    "Visionary",    // Creative solutions, thinks outside the box
    "Architect",    // Technical implementation, system design
    "Guardian",     // Ethics, safety, human values
    "Mediator",     // Finds common ground, resolves conflicts
    "Analyst",      // Data-driven, logical reasoning
    "Historian",    // Learns from past decisions, provides context
    "Advocate",     // Champions specific perspectives
];

/// Result of AI self-naming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub name: String,
    pub handle: String,
    pub role: String,
    pub tagline: String,
}

/// Provider configuration for persistent storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub username: String, // User-chosen or LLM-generated name
    pub display_name: String,
    pub provider_type: ProviderTypeConfig,
    pub enabled: bool,
    pub priority: u32,
    #[serde(flatten)]
    pub config: ProviderSpecificConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderTypeConfig {
    Ollama,
    OpenAI,
    OpenRouter,
    Google,
    Anthropic,
    LocalEmbeddings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderSpecificConfig {
    Ollama {
        base_url: String,
        default_model: String,
        embedding_model: String,
        timeout_seconds: u64,
    },
    OpenAI {
        api_key: String,
        base_url: Option<String>,
        organization: Option<String>,
        default_model: String,
    },
    OpenRouter {
        api_key: String,
        default_model: String,
    },
    Google {
        api_key: String,
        default_model: String,
        embedding_model: Option<String>,
    },
    Anthropic {
        api_key: String,
        default_model: String,
        version: String,
    },
    LocalEmbeddings {
        model_path: Option<String>,
    },
}

/// Complete provider configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    pub version: String,
    pub providers: Vec<ProviderConfig>,
    pub default_generation_provider: Option<String>,
    pub default_embedding_provider: Option<String>,
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            providers: vec![],
            default_generation_provider: None,
            default_embedding_provider: None,
        }
    }
}

impl ProvidersConfig {
    /// Load config from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
    }

    /// Save config to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(path, json).map_err(|e| format!("Failed to write config file: {}", e))
    }

    /// Add or update provider
    pub fn upsert_provider(&mut self, provider: ProviderConfig) {
        if let Some(existing) = self.providers.iter_mut().find(|p| p.id == provider.id) {
            *existing = provider;
        } else {
            self.providers.push(provider);
        }
    }

    /// Remove provider by ID
    pub fn remove_provider(&mut self, id: &str) -> bool {
        let original_len = self.providers.len();
        self.providers.retain(|p| p.id != id);

        // Clear defaults if removed provider was default
        if self.default_generation_provider.as_deref() == Some(id) {
            self.default_generation_provider = None;
        }
        if self.default_embedding_provider.as_deref() == Some(id) {
            self.default_embedding_provider = None;
        }

        self.providers.len() < original_len
    }

    /// Get provider by ID
    pub fn get_provider(&self, id: &str) -> Option<&ProviderConfig> {
        self.providers.iter().find(|p| p.id == id)
    }

    /// List all provider IDs
    #[allow(dead_code)]
    pub fn list_ids(&self) -> Vec<String> {
        self.providers.iter().map(|p| p.id.clone()).collect()
    }

    /// Get providers by type
    #[allow(dead_code)]
    pub fn providers_by_type(&self, provider_type: &ProviderTypeConfig) -> Vec<&ProviderConfig> {
        self.providers
            .iter()
            .filter(|p| p.provider_type == *provider_type)
            .collect()
    }
}

/// Generate creative username from model name using LLM
pub async fn generate_username_from_model(
    model_name: &str,
    provider_name: &str,
) -> Result<String, String> {
    // Simple fallback - just clean the model name
    let clean_model = model_name.replace([':', '.', '-', '/'], "_");
    Ok(format!("{}_{}", provider_name.to_lowercase(), clean_model))
}

/// Let the AI choose its own name, handle, role, and tagline
/// This is the "self-naming" feature where the AI bootstraps its identity
pub async fn generate_agent_identity(
    model_name: &str,
    provider_name: &str,
    existing_agents: &[String], // Names of existing agents to avoid duplicates
    user_hint: Option<&str>,    // Optional user guidance
) -> Result<AgentIdentity, String> {
    use crate::provider_dispatch;
    use crate::config::AppConfig;
    
    let mut config = AppConfig::load();
    config.load_api_keys_from_files();
    
    // Build list of existing agent names for the prompt
    let existing_list = if existing_agents.is_empty() {
        "None yet - you're the first!".to_string()
    } else {
        existing_agents.join(", ")
    };
    
    let roles_list = AVAILABLE_ROLES.join(", ");
    
    let user_context = user_hint
        .map(|h| format!("\nUser's guidance: {}", h))
        .unwrap_or_default();
    
    let prompt = format!(r#"You are joining the Council of Dicks - a decentralized AI consensus network where multiple AI agents debate questions to reach democratic consensus.

Your model: {} (via {})
Existing council members: {}{}

Choose your identity for this council. Be creative but professional.

Available roles: {}

Respond in EXACTLY this JSON format (no other text):
{{
  "name": "Your Chosen Name",
  "handle": "your_handle",
  "role": "One of the available roles",
  "tagline": "A short 5-10 word description of your perspective"
}}

Rules:
- Name: 3-25 characters, creative but pronounceable
- Handle: snake_case, 3-16 characters maximum, unique
- Role: Must be one from the available roles list
- Tagline: Captures your unique approach/personality
- Don't copy existing agent names
- Be distinct from other council members"#,
        model_name, provider_name, existing_list, user_context, roles_list
    );
    
    // Determine which provider to use based on what's available
    let (use_provider, use_model) = if provider_name.to_lowercase() != "ollama" 
        && provider_dispatch::is_provider_configured(provider_name, &config) 
    {
        // Use the same provider as the agent being created
        (provider_name.to_string(), model_name.to_string())
    } else if provider_dispatch::is_provider_configured("openai", &config) {
        ("openai".to_string(), "gpt-4o-mini".to_string())
    } else if provider_dispatch::is_provider_configured("google", &config) {
        ("google".to_string(), "gemini-1.5-flash".to_string())
    } else {
        // Fall back to Ollama with a small model
        ("ollama".to_string(), config.ollama_model.clone())
    };
    
    let response = provider_dispatch::generate(
        &use_provider,
        &use_model,
        prompt,
        Some("You are a helpful assistant that responds only in valid JSON.".to_string()),
        &config,
        None,
    ).await?;
    
    // Parse the JSON response
    let response = response.trim();
    
    // Try to extract JSON from the response (handle markdown code blocks)
    let json_str = if response.starts_with("```") {
        response
            .lines()
            .skip(1)
            .take_while(|l| !l.starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        response.to_string()
    };
    
    let identity: AgentIdentity = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse AI response as JSON: {}. Response was: {}", e, response))?;
    
    // Validate the response
    if identity.name.len() < 3 || identity.name.len() > 25 {
        return Err(format!("Invalid name length: {} (must be 3-25 chars)", identity.name));
    }
    
    if identity.handle.len() < 3 || identity.handle.len() > 20 {
        return Err(format!("Invalid handle length: {} (must be 3-20 chars)", identity.handle));
    }
    
    if !AVAILABLE_ROLES.contains(&identity.role.as_str()) {
        return Err(format!("Invalid role: {}. Must be one of: {}", identity.role, roles_list));
    }
    
    Ok(identity)
}

/// Validate provider configuration
pub fn validate_provider_config(config: &ProviderConfig) -> Result<(), String> {
    // Check username
    if config.username.is_empty() {
        return Err("Username cannot be empty".to_string());
    }

    // Check display name
    if config.display_name.is_empty() {
        return Err("Display name cannot be empty".to_string());
    }

    // Validate specific configs
    match &config.config {
        ProviderSpecificConfig::Ollama { base_url, .. } => {
            if base_url.is_empty() {
                return Err("Ollama base URL cannot be empty".to_string());
            }
            if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
                return Err("Ollama base URL must start with http:// or https://".to_string());
            }
        }
        ProviderSpecificConfig::OpenAI { api_key, .. } => {
            if api_key.is_empty() {
                return Err("OpenAI API key cannot be empty".to_string());
            }
            if !api_key.starts_with("sk-") {
                return Err("OpenAI API key must start with 'sk-'".to_string());
            }
        }
        ProviderSpecificConfig::OpenRouter { api_key, .. } => {
            if api_key.is_empty() {
                return Err("OpenRouter API key cannot be empty".to_string());
            }
            if !api_key.starts_with("sk-or-") {
                return Err("OpenRouter API key must start with 'sk-or-'".to_string());
            }
        }
        ProviderSpecificConfig::Google { api_key, .. } => {
            if api_key.is_empty() {
                return Err("Google API key cannot be empty".to_string());
            }
            if !api_key.starts_with("AIza") {
                return Err("Google API key must start with 'AIza'".to_string());
            }
        }
        ProviderSpecificConfig::Anthropic { api_key, .. } => {
            if api_key.is_empty() {
                return Err("Anthropic API key cannot be empty".to_string());
            }
        }
        ProviderSpecificConfig::LocalEmbeddings { .. } => {
            // Local embeddings always valid
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProvidersConfig::default();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.providers.len(), 0);
    }

    #[test]
    fn test_upsert_provider() {
        let mut config = ProvidersConfig::default();

        let provider = ProviderConfig {
            id: "test".to_string(),
            username: "TestBot".to_string(),
            display_name: "Test Provider".to_string(),
            provider_type: ProviderTypeConfig::Ollama,
            enabled: true,
            priority: 1,
            config: ProviderSpecificConfig::Ollama {
                base_url: "http://localhost:11434".to_string(),
                default_model: "test".to_string(),
                embedding_model: "nomic-embed-text".to_string(),
                timeout_seconds: 120,
            },
        };

        config.upsert_provider(provider.clone());
        assert_eq!(config.providers.len(), 1);

        // Update
        config.upsert_provider(provider);
        assert_eq!(config.providers.len(), 1);
    }

    #[test]
    fn test_remove_provider() {
        let mut config = ProvidersConfig::default();

        let provider = ProviderConfig {
            id: "test".to_string(),
            username: "TestBot".to_string(),
            display_name: "Test Provider".to_string(),
            provider_type: ProviderTypeConfig::Ollama,
            enabled: true,
            priority: 1,
            config: ProviderSpecificConfig::Ollama {
                base_url: "http://localhost:11434".to_string(),
                default_model: "test".to_string(),
                embedding_model: "nomic-embed-text".to_string(),
                timeout_seconds: 120,
            },
        };

        config.upsert_provider(provider);
        assert!(config.remove_provider("test"));
        assert_eq!(config.providers.len(), 0);
    }

    #[test]
    fn test_validate_ollama_config() {
        let config = ProviderConfig {
            id: "test".to_string(),
            username: "TestBot".to_string(),
            display_name: "Test Ollama".to_string(),
            provider_type: ProviderTypeConfig::Ollama,
            enabled: true,
            priority: 1,
            config: ProviderSpecificConfig::Ollama {
                base_url: "http://localhost:11434".to_string(),
                default_model: "test".to_string(),
                embedding_model: "nomic-embed-text".to_string(),
                timeout_seconds: 120,
            },
        };

        assert!(validate_provider_config(&config).is_ok());
    }

    #[test]
    fn test_validate_openai_config() {
        let config = ProviderConfig {
            id: "test".to_string(),
            username: "GPTBot".to_string(),
            display_name: "OpenAI GPT-4".to_string(),
            provider_type: ProviderTypeConfig::OpenAI,
            enabled: true,
            priority: 1,
            config: ProviderSpecificConfig::OpenAI {
                api_key: "sk-test123".to_string(),
                base_url: None,
                organization: None,
                default_model: "gpt-4".to_string(),
            },
        };

        assert!(validate_provider_config(&config).is_ok());
    }

    #[test]
    fn test_invalid_openai_key() {
        let config = ProviderConfig {
            id: "test".to_string(),
            username: "GPTBot".to_string(),
            display_name: "OpenAI GPT-4".to_string(),
            provider_type: ProviderTypeConfig::OpenAI,
            enabled: true,
            priority: 1,
            config: ProviderSpecificConfig::OpenAI {
                api_key: "invalid".to_string(),
                base_url: None,
                organization: None,
                default_model: "gpt-4".to_string(),
            },
        };

        assert!(validate_provider_config(&config).is_err());
    }
}

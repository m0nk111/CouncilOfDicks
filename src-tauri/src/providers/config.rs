use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
    // For now, generate a simple username
    // TODO: Use LLM to generate creative names
    let clean_model = model_name.replace([':', '.'], "_");
    Ok(format!("{}_{}", provider_name.to_lowercase(), clean_model))
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

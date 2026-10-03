use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ollama_url: String,
    pub ollama_model: String,
    pub ollama_username: Option<String>,
    pub ollama_password: Option<String>,
    #[serde(default)]
    pub guardian_url: Option<String>,
    #[serde(default)]
    pub guardian_model: Option<String>,
    #[serde(default)]
    pub guardian_api_key: Option<String>,
    #[serde(default = "default_generation_provider_name")]
    pub default_generation_provider: String,
    #[serde(default)]
    pub default_generation_model: Option<String>,
    #[serde(default)]
    pub openai_base_url: Option<String>,
    pub debug_enabled: bool,
    pub initial_topic: Option<String>,
    pub topic_interval: u64,
    pub p2p_port: u16,
    pub bootstrap_peers: Vec<String>,
    pub user_handle: String,
    pub question_generation_prompt: String,

    // ChatBot relevance gating
    /// If true, ChatBot will always respond (dev/testing mode).
    #[serde(default)]
    pub chat_force_relevance: bool,
    /// If true, ChatBot may respond to AI-authored messages (with safeguards).
    #[serde(default)]
    pub chat_allow_ai_to_ai: bool,
    /// Maximum number of AI-triggered follow-ups after a human message.
    /// This prevents infinite ping-pong between agents.
    #[serde(default = "default_chat_ai_to_ai_budget")]
    pub chat_ai_to_ai_budget: u32,
    /// DANGEROUS: if true, disables the AI-to-AI budget limiter.
    /// This can cause infinite loops if `chat_allow_ai_to_ai` is also true.
    #[serde(default)]
    pub chat_ai_to_ai_infinite_loop: bool,
    /// Threshold in [0.0, 1.0]. Agent responds if relevance_score >= threshold.
    #[serde(default = "default_chat_relevance_threshold")]
    pub chat_relevance_threshold: f32,
    // Provider API keys
    #[serde(default)]
    pub openai_api_key: Option<String>,
    #[serde(default)]
    pub openrouter_api_key: Option<String>,
    #[serde(default)]
    pub google_api_key: Option<String>,
}

fn default_chat_relevance_threshold() -> f32 {
    0.55
}

fn default_chat_ai_to_ai_budget() -> u32 {
    16
}

fn default_generation_provider_name() -> String {
    "guardian".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://192.168.1.5:11434".to_string(),
            ollama_model: "qwen3-coder-30b-q4km-32k:latest".to_string(),
            ollama_username: None,
            ollama_password: None,
            guardian_url: Some("http://127.0.0.1:11434".to_string()),
            guardian_model: None,
            guardian_api_key: None,
            default_generation_provider: default_generation_provider_name(),
            default_generation_model: None,
            openai_base_url: None,
            debug_enabled: true,
            initial_topic: Some("The Future of AI".to_string()),
            topic_interval: 300,
            p2p_port: 9000,
            bootstrap_peers: vec![],
            user_handle: "human_user".to_string(),
            question_generation_prompt: "Generate a single, short, provocative, and open-ended philosophical or ethical question for an AI council to debate. The question should be deep and require nuanced thinking. Do not include any preamble, explanation, or quotes. Just the question itself.".to_string(),
            chat_force_relevance: false,
            chat_allow_ai_to_ai: false,
            chat_ai_to_ai_budget: default_chat_ai_to_ai_budget(),
            chat_ai_to_ai_infinite_loop: false,
            chat_relevance_threshold: default_chat_relevance_threshold(),
            openai_api_key: None,
            openrouter_api_key: None,
            google_api_key: None,
        }
    }
}

impl AppConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_debug(mut self, enabled: bool) -> Self {
        self.debug_enabled = enabled;
        self
    }

    pub fn get_config_path() -> PathBuf {
        let mut path = PathBuf::from("config/app_config.json");
        if !path.exists() {
            // Try parent directory (for when running from src-tauri)
            let parent_path = PathBuf::from("../config/app_config.json");
            // If parent exists or if we are in src-tauri (check for Cargo.toml), use parent
            if parent_path.exists() || PathBuf::from("Cargo.toml").exists() {
                path = parent_path;
            }
        }
        path
    }

    pub fn load() -> Self {
        let path = Self::get_config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => eprintln!("Failed to parse config: {}", e),
                },
                Err(e) => eprintln!("Failed to read config file: {}", e),
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_config_path();
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                let _ = fs::create_dir_all(parent);
            }
        }

        match serde_json::to_string_pretty(self) {
            Ok(content) => fs::write(path, content).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn resolved_default_generation_provider(&self) -> &str {
        let provider = self.default_generation_provider.trim();
        if provider.is_empty() {
            "guardian"
        } else {
            provider
        }
    }

    pub fn resolved_guardian_base_url(&self) -> String {
        let base = self
            .guardian_url
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| self.ollama_url.trim());

        let trimmed = base.trim_end_matches('/');
        if trimmed.ends_with("/v1") {
            trimmed.to_string()
        } else {
            format!("{}/v1", trimmed)
        }
    }

    pub fn resolved_guardian_legacy_url(&self) -> String {
        self.resolved_guardian_base_url()
            .trim_end_matches("/v1")
            .to_string()
    }

    pub fn resolved_guardian_model(&self) -> String {
        self.guardian_model
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| self.ollama_model.clone())
    }

    pub fn resolved_default_generation_model(&self) -> String {
        let default_model = self.default_generation_model
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| {
                if self.resolved_default_generation_provider().eq_ignore_ascii_case("guardian") {
                    self.resolved_guardian_model()
                } else {
                    self.ollama_model.clone()
                }
            });

        default_model
    }

    /// Load API keys from ~/.secrets/keys/ directory
    /// Files: guardian.key, openai.key, google.key, openrouter.key
    pub fn load_api_keys_from_files(&mut self) {
        let keys_dir = dirs::home_dir()
            .map(|h| h.join(".secrets/keys"))
            .unwrap_or_else(|| PathBuf::from("~/.secrets/keys"));

        if self.guardian_api_key.is_none() {
            if let Ok(key) = fs::read_to_string(keys_dir.join("guardian.key")) {
                let key = key.trim().to_string();
                if !key.is_empty() {
                    self.guardian_api_key = Some(key);
                }
            }
        }

        if self.guardian_api_key.is_none() {
            #[derive(Debug, Deserialize)]
            struct GuardianKeyRecord {
                name: String,
                #[serde(default)]
                metadata: HashMap<String, String>,
            }

            let guardian_key_path = dirs::home_dir()
                .map(|home| home.join("llama_cpp_guardian/config/api_keys.json"));

            if let Some(path) = guardian_key_path {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(records) = serde_json::from_str::<HashMap<String, GuardianKeyRecord>>(&content) {
                        let handle = self.user_handle.trim();
                        if let Some((token, _)) = records.iter().find(|(_, record)| {
                            record.name == handle
                                || record
                                    .metadata
                                    .get("client")
                                    .map(|client| client == handle)
                                    .unwrap_or(false)
                        }) {
                            self.guardian_api_key = Some(token.clone());
                        }
                    }
                }
            }
        }

        // OpenAI
        if self.openai_api_key.is_none() {
            if let Ok(key) = fs::read_to_string(keys_dir.join("openai.key")) {
                let key = key.trim().to_string();
                if !key.is_empty() {
                    self.openai_api_key = Some(key);
                }
            }
        }

        // Google
        if self.google_api_key.is_none() {
            if let Ok(key) = fs::read_to_string(keys_dir.join("google.key")) {
                let key = key.trim().to_string();
                if !key.is_empty() {
                    self.google_api_key = Some(key);
                }
            }
        }

        // OpenRouter
        if self.openrouter_api_key.is_none() {
            if let Ok(key) = fs::read_to_string(keys_dir.join("openrouter.key")) {
                let key = key.trim().to_string();
                if !key.is_empty() {
                    self.openrouter_api_key = Some(key);
                }
            }
        }
    }

    /// Get configured provider names (for display)
    pub fn available_providers(&self) -> Vec<&str> {
        let mut providers = vec!["ollama"];
        if self.guardian_api_key.is_some() {
            providers.push("guardian");
        }
        if self.openai_api_key.is_some() {
            providers.push("openai");
        }
        if self.google_api_key.is_some() {
            providers.push("google");
        }
        if self.openrouter_api_key.is_some() {
            providers.push("openrouter");
        }
        providers
    }
}

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ollama_url: String,
    pub ollama_model: String,
    pub ollama_username: Option<String>,
    pub ollama_password: Option<String>,
    pub debug_enabled: bool,
    pub initial_topic: Option<String>,
    pub topic_interval: u64,
    pub p2p_port: u16,
    pub bootstrap_peers: Vec<String>,
    pub user_handle: String,
    pub question_generation_prompt: String,
    // Provider API keys
    #[serde(default)]
    pub openai_api_key: Option<String>,
    #[serde(default)]
    pub openrouter_api_key: Option<String>,
    #[serde(default)]
    pub google_api_key: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://192.168.1.5:11434".to_string(),
            ollama_model: "qwen3-coder-30b-q4km-32k:latest".to_string(),
            ollama_username: None,
            ollama_password: None,
            debug_enabled: true,
            initial_topic: Some("The Future of AI".to_string()),
            topic_interval: 300,
            p2p_port: 9000,
            bootstrap_peers: vec![],
            user_handle: "human_user".to_string(),
            question_generation_prompt: "Generate a single, short, provocative, and open-ended philosophical or ethical question for an AI council to debate. The question should be deep and require nuanced thinking. Do not include any preamble, explanation, or quotes. Just the question itself.".to_string(),
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

    /// Load API keys from ~/.secrets/keys/ directory
    /// Files: openai.key, google.key, openrouter.key
    pub fn load_api_keys_from_files(&mut self) {
        let keys_dir = dirs::home_dir()
            .map(|h| h.join(".secrets/keys"))
            .unwrap_or_else(|| PathBuf::from("~/.secrets/keys"));

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
        let mut providers = vec!["ollama"]; // Always available
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

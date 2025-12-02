use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ollama_url: String,
    pub ollama_model: String,
    pub debug_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://192.168.1.5:11434".to_string(),
            ollama_model: "mistral:7b".to_string(),
            debug_enabled: true,
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
}

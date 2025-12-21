// Provider dispatcher - Routes generation requests to the appropriate AI provider

use crate::config::AppConfig;
use crate::ollama;
use crate::providers::{AIProvider, GenerationRequest, GoogleProvider, OpenAIProvider};
use crate::logger::Logger;
use std::sync::Arc;

/// Generate text using the specified provider
/// 
/// # Arguments
/// * `provider` - Provider name: "ollama", "openai", "openrouter", "google"
/// * `model` - Model name (e.g., "gpt-4o", "gemini-1.5-flash", "qwen2.5:7b")
/// * `prompt` - The user prompt/message
/// * `system_prompt` - Optional system prompt
/// * `config` - App configuration with API keys
/// * `logger` - Logger for debug output
pub async fn generate(
    provider: &str,
    model: &str,
    prompt: String,
    system_prompt: Option<String>,
    config: &AppConfig,
    logger: Option<Arc<Logger>>,
) -> Result<String, String> {
    generate_with_timeout(provider, model, prompt, system_prompt, config, logger, None).await
}

/// Generate text with custom timeout (for slow models)
pub async fn generate_with_timeout(
    provider: &str,
    model: &str,
    prompt: String,
    system_prompt: Option<String>,
    config: &AppConfig,
    logger: Option<Arc<Logger>>,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    match provider.to_lowercase().as_str() {
        "ollama" => {
            let auth = if let (Some(u), Some(p)) = (&config.ollama_username, &config.ollama_password) {
                Some((u.as_str(), p.as_str()))
            } else {
                None
            };

            ollama::ask_ollama_with_timeout(
                &config.ollama_url,
                model,
                prompt,
                system_prompt,
                auth,
                timeout_secs,
            )
            .await
        }

        "openai" => {
            let api_key = config.openai_api_key.as_ref()
                .ok_or_else(|| "OpenAI API key not configured".to_string())?;

            let log = logger.unwrap_or_else(|| Arc::new(Logger::new(false)));
            let provider = OpenAIProvider::new(
                api_key.clone(),
                model.to_string(),
                log,
            );

            let request = GenerationRequest {
                model: model.to_string(),
                prompt,
                system_prompt,
                temperature: 0.7,
                max_tokens: None,
                stream: false,
            };

            match provider.generate(request).await {
                Ok(response) => Ok(response.text),
                Err(e) => Err(e.to_string()),
            }
        }

        "openrouter" => {
            let api_key = config.openrouter_api_key.as_ref()
                .ok_or_else(|| "OpenRouter API key not configured".to_string())?;

            let log = logger.unwrap_or_else(|| Arc::new(Logger::new(false)));
            let provider = OpenAIProvider::openrouter(
                api_key.clone(),
                model.to_string(),
                log,
            );

            let request = GenerationRequest {
                model: model.to_string(),
                prompt,
                system_prompt,
                temperature: 0.7,
                max_tokens: None,
                stream: false,
            };

            match provider.generate(request).await {
                Ok(response) => Ok(response.text),
                Err(e) => Err(e.to_string()),
            }
        }

        "google" => {
            let api_key = config.google_api_key.as_ref()
                .ok_or_else(|| "Google API key not configured".to_string())?;

            let log = logger.unwrap_or_else(|| Arc::new(Logger::new(false)));
            let provider = GoogleProvider::new(
                api_key.clone(),
                model.to_string(),
                log,
            );

            let request = GenerationRequest {
                model: model.to_string(),
                prompt,
                system_prompt,
                temperature: 0.7,
                max_tokens: None,
                stream: false,
            };

            match provider.generate(request).await {
                Ok(response) => Ok(response.text),
                Err(e) => Err(e.to_string()),
            }
        }

        _ => Err(format!("Unknown provider: {}", provider)),
    }
}

/// Helper to check if a provider is configured
pub fn is_provider_configured(provider: &str, config: &AppConfig) -> bool {
    match provider.to_lowercase().as_str() {
        "ollama" => true, // Always available (may fail at runtime, but configured)
        "openai" => config.openai_api_key.is_some(),
        "openrouter" => config.openrouter_api_key.is_some(),
        "google" => config.google_api_key.is_some(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_provider_configured() {
        let config = AppConfig::default();
        
        assert!(is_provider_configured("ollama", &config));
        assert!(!is_provider_configured("openai", &config));
        assert!(!is_provider_configured("google", &config));
        assert!(!is_provider_configured("openrouter", &config));
    }

    #[test]
    fn test_is_provider_configured_with_keys() {
        let mut config = AppConfig::default();
        config.openai_api_key = Some("sk-test".to_string());
        config.google_api_key = Some("AIza-test".to_string());
        
        assert!(is_provider_configured("openai", &config));
        assert!(is_provider_configured("google", &config));
        assert!(!is_provider_configured("openrouter", &config));
    }
}

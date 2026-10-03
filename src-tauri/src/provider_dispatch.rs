// Provider dispatcher - Routes generation requests to the appropriate AI provider

use crate::config::AppConfig;
use crate::ollama;
use crate::providers::{AIProvider, GenerationRequest, GoogleProvider, OpenAIProvider};
use crate::logger::Logger;
use std::sync::Arc;
use std::time::Duration;

fn build_openai_compatible_provider(
    api_key: String,
    base_url: String,
    model: &str,
    provider_name: &str,
    logger: Option<Arc<Logger>>,
    timeout_secs: Option<u64>,
) -> OpenAIProvider {
    let log = logger.unwrap_or_else(|| Arc::new(Logger::new(false)));
    let provider = OpenAIProvider::with_base_url(
        api_key,
        base_url,
        model.to_string(),
        provider_name.to_string(),
        log,
    );

    if let Some(timeout) = timeout_secs {
        provider.with_timeout(Duration::from_secs(timeout))
    } else {
        provider
    }
}

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

/// Generate text using the configured default provider/model.
pub async fn generate_with_default(
    prompt: String,
    system_prompt: Option<String>,
    config: &AppConfig,
    logger: Option<Arc<Logger>>,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let provider = config.resolved_default_generation_provider();
    let model = config.resolved_default_generation_model();
    generate_with_timeout(provider, &model, prompt, system_prompt, config, logger, timeout_secs)
        .await
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
        "guardian" => {
            let api_key = config
                .guardian_api_key
                .as_ref()
                .ok_or_else(|| "Guardian API key not configured".to_string())?;

            let provider = build_openai_compatible_provider(
                api_key.clone(),
                config.resolved_guardian_base_url(),
                model,
                "Guardian",
                logger,
                timeout_secs,
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

        "ollama" => {
            // Ollama Guardian uses username-only auth (app name), password is optional
            let auth = config.ollama_username.as_ref().map(|u| {
                (u.as_str(), config.ollama_password.as_deref().unwrap_or(""))
            });

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

            let provider = if let Some(base_url) = config
                .openai_base_url
                .as_ref()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
            {
                build_openai_compatible_provider(
                    api_key.clone(),
                    base_url.to_string(),
                    model,
                    "OpenAI-Compatible",
                    logger,
                    timeout_secs,
                )
            } else {
                build_openai_compatible_provider(
                    api_key.clone(),
                    "https://api.openai.com/v1".to_string(),
                    model,
                    "OpenAI",
                    logger,
                    timeout_secs,
                )
            };

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

            let provider = build_openai_compatible_provider(
                api_key.clone(),
                "https://openrouter.ai/api/v1".to_string(),
                model,
                "OpenRouter",
                logger,
                timeout_secs,
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
        "guardian" => config.guardian_api_key.is_some(),
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
        assert!(!is_provider_configured("guardian", &config));
        assert!(!is_provider_configured("openai", &config));
        assert!(!is_provider_configured("google", &config));
        assert!(!is_provider_configured("openrouter", &config));
    }

    #[test]
    fn test_is_provider_configured_with_keys() {
        let mut config = AppConfig::default();
        config.guardian_api_key = Some("flip-test".to_string());
        config.openai_api_key = Some("sk-test".to_string());
        config.google_api_key = Some("AIza-test".to_string());
        
        assert!(is_provider_configured("guardian", &config));
        assert!(is_provider_configured("openai", &config));
        assert!(is_provider_configured("google", &config));
        assert!(!is_provider_configured("openrouter", &config));
    }
}

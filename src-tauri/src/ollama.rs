use crate::config::AppConfig;
use crate::logger::Logger;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaTagResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaModelInfo {
    name: String,
}

/// Ollama client for making API requests
pub struct OllamaClient {
    config: AppConfig,
    #[allow(dead_code)]
    logger: Arc<Logger>,
}

impl OllamaClient {
    pub fn new(config: AppConfig, logger: Arc<Logger>) -> Self {
        Self { config, logger }
    }

    pub async fn ask(&self, model: &str, prompt: &str, system: Option<&str>) -> Result<String, String> {
        // Ollama Guardian uses username-only auth (app name), password is optional
        let auth = self.config.ollama_username.as_ref().map(|u| {
            (u.as_str(), self.config.ollama_password.as_deref().unwrap_or(""))
        });
        ask_ollama_with_auth(&self.config.ollama_url, model, prompt.to_string(), system.map(|s| s.to_string()), auth).await
    }
}

/// Internal function for HTTP API use
pub async fn ask_ollama_internal(
    state: &crate::state::AppState,
    model: String,
    prompt: String,
    system: Option<String>,
) -> Result<String, String> {
    let config = state.get_config();
    // Ollama Guardian uses username-only auth (app name), password is optional
    let auth = config.ollama_username.as_ref().map(|u| {
        (u.as_str(), config.ollama_password.as_deref().unwrap_or(""))
    });
    ask_ollama_with_auth(&config.ollama_url, &model, prompt, system, auth).await
}

pub async fn ask_ollama(url: &str, model: &str, prompt: String) -> Result<String, String> {
    ask_ollama_with_timeout(url, model, prompt, None, None, None).await
}

pub async fn ask_ollama_with_auth(
    url: &str,
    model: &str,
    prompt: String,
    system: Option<String>,
    basic_auth: Option<(&str, &str)>,
) -> Result<String, String> {
    ask_ollama_with_timeout(url, model, prompt, system, basic_auth, None).await
}

/// Ask Ollama with custom timeout (for slow models like deepseek-r1)
pub async fn ask_ollama_with_timeout(
    url: &str,
    model: &str,
    prompt: String,
    system: Option<String>,
    basic_auth: Option<(&str, &str)>,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let timeout = timeout_secs.unwrap_or(OLLAMA_DEFAULT_TIMEOUT_SECS);
    
    println!("🔍 [DEBUG] Asking Ollama: {}", prompt);
    println!("📡 [DEBUG] URL: {}, Model: {}, Timeout: {}s", url, model, timeout);

    let base_url = url.trim_end_matches('/');
    let client = build_http_client_with_timeout(timeout)?;
    let resolved_model = resolve_model(&client, base_url, model, basic_auth).await?;

    if resolved_model != model {
        println!(
            "⚠️ [OLLAMA] Requested model '{}' missing. Falling back to '{}'",
            model, resolved_model
        );
    }

    let endpoint = format!("{}/api/generate", base_url);
    let request_body = OllamaRequest {
        model: resolved_model.clone(),
        prompt,
        system,
        stream: false,
    };

    let mut request = client.post(&endpoint).json(&request_body);

    // Add basic auth if provided
    if let Some((username, password)) = basic_auth {
        request = request.basic_auth(username, Some(password));
    }

    println!("⏳ [OLLAMA] Sending request to {} (timeout: {}s)...", resolved_model, timeout);
    
    let response = request
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                format!(
                    "⏱️ Ollama request timed out after {}s. Model '{}' may need more time. \
                    Set timeout_secs in agent config for slower models.",
                    timeout, resolved_model
                )
            } else if e.is_connect() {
                format!("❌ Failed to connect to Ollama at {}: Is Ollama running?", url)
            } else {
                format!("❌ Ollama request failed: {}", e)
            }
        })?;

    if !response.status().is_success() {
        return Err(format!(
            "❌ Ollama returned error status: {}",
            response.status()
        ));
    }

    // First get the raw text to check for errors
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("❌ Failed to read Ollama response body: {}", e))?;
    
    // Check for common error responses
    if response_text.contains("Not authenticated") {
        return Err(format!(
            "❌ Ollama requires authentication. Check ollama_username/ollama_password in config. Model: {}",
            model
        ));
    }
    
    // Try to parse the JSON
    let ollama_response: OllamaResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!(
            "❌ Failed to parse Ollama response: {}. Raw: {}",
            e, 
            &response_text[..response_text.len().min(200)]
        ))?;
    
    // Check for empty response
    if ollama_response.response.trim().is_empty() {
        return Err(format!(
            "❌ Model '{}' returned empty response. This model may have crashed or doesn't support this prompt type.",
            model
        ));
    }

    println!("✅ [DEBUG] Got response from Ollama!");
    Ok(ollama_response.response)
}

/// Default timeout for Ollama requests (5 minutes)
/// Large models like deepseek-r1:32b can take a long time to generate
pub const OLLAMA_DEFAULT_TIMEOUT_SECS: u64 = 300;

fn build_http_client_with_timeout(timeout_secs: u64) -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

fn build_http_client() -> Result<Client, String> {
    build_http_client_with_timeout(OLLAMA_DEFAULT_TIMEOUT_SECS)
}

async fn resolve_model(
    client: &Client,
    base_url: &str,
    requested_model: &str,
    basic_auth: Option<(&str, &str)>,
) -> Result<String, String> {
    let available = fetch_available_models(client, base_url, basic_auth).await?;

    if available.is_empty() {
        return Err("❌ No models available on Ollama server".to_string());
    }

    if requested_model.is_empty() {
        println!("⚠️ [OLLAMA] No model configured, using '{}'", available[0]);
        return Ok(available[0].clone());
    }

    if let Some(match_exact) = available
        .iter()
        .find(|name| name.as_str() == requested_model)
    {
        return Ok(match_exact.clone());
    }

    if let Some(match_prefix) = available
        .iter()
        .find(|name| name.starts_with(requested_model))
    {
        println!(
            "⚠️ [OLLAMA] Requested model '{}' not found exactly. Using closest match '{}'.",
            requested_model, match_prefix
        );
        return Ok(match_prefix.clone());
    }

    println!(
        "⚠️ [OLLAMA] Requested model '{}' unavailable. Using '{}' from available list.",
        requested_model, available[0]
    );
    Ok(available[0].clone())
}

async fn fetch_available_models(
    client: &Client,
    base_url: &str,
    basic_auth: Option<(&str, &str)>,
) -> Result<Vec<String>, String> {
    let endpoint = format!("{}/api/tags", base_url);
    let mut request = client.get(&endpoint);

    if let Some((username, password)) = basic_auth {
        request = request.basic_auth(username, Some(password));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("❌ Failed to query Ollama tags at {}: {}", base_url, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "❌ Ollama tags endpoint returned status: {}",
            response.status()
        ));
    }

    let tag_response: OllamaTagResponse = response
        .json()
        .await
        .map_err(|e| format!("❌ Failed to parse Ollama tag response: {}", e))?;

    Ok(tag_response.models.into_iter().map(|m| m.name).collect())
}

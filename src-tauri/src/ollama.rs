use crate::config::AppConfig;
use crate::logger::Logger;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
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
    logger: Arc<Logger>,
}

impl OllamaClient {
    pub fn new(config: AppConfig, logger: Arc<Logger>) -> Self {
        Self { config, logger }
    }

    pub async fn ask(&self, model: &str, prompt: &str) -> Result<String, String> {
        ask_ollama(&self.config.ollama_url, model, prompt.to_string()).await
    }
}

/// Internal function for HTTP API use
pub async fn ask_ollama_internal(
    state: &crate::state::AppState,
    model: String,
    prompt: String,
) -> Result<String, String> {
    let config = state.get_config();
    ask_ollama(&config.ollama_url, &model, prompt).await
}

pub async fn ask_ollama(url: &str, model: &str, prompt: String) -> Result<String, String> {
    ask_ollama_with_auth(url, model, prompt, Some(("CouncilOfDicks", ""))).await
}

pub async fn ask_ollama_with_auth(
    url: &str,
    model: &str,
    prompt: String,
    basic_auth: Option<(&str, &str)>,
) -> Result<String, String> {
    println!("🔍 [DEBUG] Asking Ollama: {}", prompt);
    println!("📡 [DEBUG] URL: {}, Model: {}", url, model);

    let base_url = url.trim_end_matches('/');
    let client = build_http_client()?;
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
        stream: false,
    };

    let mut request = client.post(&endpoint).json(&request_body);

    // Add basic auth if provided
    if let Some((username, password)) = basic_auth {
        request = request.basic_auth(username, Some(password));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("❌ Failed to connect to Ollama at {}: {}", url, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "❌ Ollama returned error status: {}",
            response.status()
        ));
    }

    let ollama_response: OllamaResponse = response
        .json()
        .await
        .map_err(|e| format!("❌ Failed to parse Ollama response: {}", e))?;

    println!("✅ [DEBUG] Got response from Ollama!");
    Ok(ollama_response.response)
}

fn build_http_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
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

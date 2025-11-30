use serde::{Deserialize, Serialize};

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

pub async fn ask_ollama(url: &str, model: &str, prompt: String) -> Result<String, String> {
    println!("🔍 [DEBUG] Asking Ollama: {}", prompt);
    println!("📡 [DEBUG] URL: {}, Model: {}", url, model);

    let endpoint = format!("{}/api/generate", url);
    let request_body = OllamaRequest {
        model: model.to_string(),
        prompt,
        stream: false,
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .post(&endpoint)
        .json(&request_body)
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

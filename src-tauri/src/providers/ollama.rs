use crate::logger::{LogLevel, Logger};
use crate::providers::{
    AIProvider, FinishReason, GenerationRequest, GenerationResponse, ModelInfo, ProviderError,
    ProviderHealth, ProviderType,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaEmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

/// Ollama AI provider implementation
pub struct OllamaProvider {
    base_url: String,
    default_model: String,
    embedding_model: String,
    timeout: Duration,
    logger: Arc<Logger>,
}

impl OllamaProvider {
    pub fn new(
        base_url: String,
        default_model: String,
        logger: Arc<Logger>,
    ) -> Self {
        logger.log(
            LogLevel::Info,
            "ollama_provider",
            &format!("📡 Initializing Ollama provider at {}", base_url),
        );

        Self {
            base_url,
            default_model,
            embedding_model: "nomic-embed-text".to_string(),
            timeout: Duration::from_secs(120),
            logger,
        }
    }

    pub fn with_embedding_model(mut self, model: String) -> Self {
        self.embedding_model = model;
        self
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    fn name(&self) -> &str {
        "Ollama"
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Network {
            requires_internet: false,
        }
    }

    async fn generate(
        &self,
        request: GenerationRequest,
    ) -> Result<GenerationResponse, ProviderError> {
        self.logger.log(
            LogLevel::Debug,
            "ollama_provider",
            &format!("🤖 Generating with model: {}", request.model),
        );

        let endpoint = format!("{}/api/generate", self.base_url);

        // Build prompt with system prompt if provided
        let full_prompt = if let Some(system) = request.system_prompt {
            format!("{}\n\n{}", system, request.prompt)
        } else {
            request.prompt.clone()
        };

        let ollama_request = OllamaRequest {
            model: request.model.clone(),
            prompt: full_prompt,
            stream: false,
        };

        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        let response = client
            .post(&endpoint)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::NetworkError(format!(
                "Ollama returned status: {}",
                response.status()
            )));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        self.logger.log(
            LogLevel::Success,
            "ollama_provider",
            &format!("✅ Generated {} chars", ollama_response.response.len()),
        );

        Ok(GenerationResponse {
            text: ollama_response.response,
            model: request.model,
            tokens_used: 0, // Ollama doesn't return token count in simple API
            finish_reason: FinishReason::Stop,
        })
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, ProviderError> {
        self.logger.log(
            LogLevel::Debug,
            "ollama_provider",
            &format!("🔢 Generating embedding for {} chars", text.len()),
        );

        let endpoint = format!("{}/api/embeddings", self.base_url);

        let embed_request = OllamaEmbeddingRequest {
            model: self.embedding_model.clone(),
            prompt: text.to_string(),
        };

        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        let response = client
            .post(&endpoint)
            .json(&embed_request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::NetworkError(format!(
                "Ollama embeddings returned status: {}",
                response.status()
            )));
        }

        let embed_response: OllamaEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        self.logger.log(
            LogLevel::Success,
            "ollama_provider",
            &format!("✅ Generated {}-dim embedding", embed_response.embedding.len()),
        );

        Ok(embed_response.embedding)
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        self.logger.log(
            LogLevel::Debug,
            "ollama_provider",
            "📋 Listing available models",
        );

        let endpoint = format!("{}/api/tags", self.base_url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        let response = client
            .get(&endpoint)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::NetworkError(format!(
                "Ollama tags returned status: {}",
                response.status()
            )));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        let models: Vec<ModelInfo> = result["models"]
            .as_array()
            .ok_or(ProviderError::InternalError("No models array".to_string()))?
            .iter()
            .filter_map(|m| {
                m["name"].as_str().map(|name| ModelInfo {
                    id: name.to_string(),
                    name: name.to_string(),
                    context_length: 8192, // Default, Ollama doesn't expose this
                    supports_embeddings: name.contains("embed"),
                    supports_function_calling: false,
                })
            })
            .collect();

        self.logger.log(
            LogLevel::Success,
            "ollama_provider",
            &format!("✅ Found {} models", models.len()),
        );

        Ok(models)
    }

    async fn health_check(&self) -> Result<ProviderHealth, ProviderError> {
        let start = Instant::now();

        let endpoint = format!("{}/api/tags", self.base_url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        match client.get(&endpoint).send().await {
            Ok(response) if response.status().is_success() => {
                let latency = start.elapsed().as_millis() as u64;
                Ok(ProviderHealth {
                    healthy: true,
                    latency_ms: Some(latency),
                    error: None,
                })
            }
            Ok(response) => Ok(ProviderHealth {
                healthy: false,
                latency_ms: None,
                error: Some(format!("Status: {}", response.status())),
            }),
            Err(e) => Ok(ProviderHealth {
                healthy: false,
                latency_ms: None,
                error: Some(e.to_string()),
            }),
        }
    }

    fn is_available(&self) -> bool {
        // Simple check - could be cached
        true
    }

    fn supports_embeddings(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn max_context_length(&self) -> usize {
        8192 // Default, should be model-specific
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::Logger;

    #[test]
    fn test_ollama_provider_creation() {
        let logger = Arc::new(Logger::new(false));
        let provider = OllamaProvider::new(
            "http://192.168.1.5:11434".to_string(),
            "qwen2.5-coder:7b".to_string(),
            logger,
        );

        assert_eq!(provider.name(), "Ollama");
        assert_eq!(
            provider.provider_type(),
            ProviderType::Network {
                requires_internet: false
            }
        );
        assert!(provider.supports_embeddings());
        assert!(provider.is_available());
    }
}

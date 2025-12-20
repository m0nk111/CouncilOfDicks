use crate::logger::{LogLevel, Logger};
use crate::providers::{
    AIProvider, FinishReason, GenerationRequest, GenerationResponse, ModelInfo, ProviderError,
    ProviderHealth, ProviderType,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============================================================================
// Google Gemini API structures
// ============================================================================

#[derive(Debug, Clone, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Debug, Clone, Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Clone, Serialize)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(rename = "maxOutputTokens", skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
    error: Option<GeminiError>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContentResponse>,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiContentResponse {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiUsageMetadata {
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiError {
    message: String,
    code: Option<i32>,
}

// ============================================================================
// Embedding structures
// ============================================================================

#[derive(Debug, Clone, Serialize)]
struct GeminiEmbedRequest {
    model: String,
    content: GeminiEmbedContent,
}

#[derive(Debug, Clone, Serialize)]
struct GeminiEmbedContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiEmbedResponse {
    embedding: Option<GeminiEmbeddingData>,
    error: Option<GeminiError>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiEmbeddingData {
    values: Vec<f32>,
}

// ============================================================================
// Models list structures
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
struct GeminiModelsResponse {
    models: Option<Vec<GeminiModelInfo>>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiModelInfo {
    name: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "inputTokenLimit")]
    input_token_limit: Option<usize>,
    #[serde(rename = "supportedGenerationMethods")]
    supported_generation_methods: Option<Vec<String>>,
}

// ============================================================================
// Google Gemini Provider Implementation
// ============================================================================

/// Google Gemini AI provider
pub struct GoogleProvider {
    api_key: String,
    default_model: String,
    embedding_model: String,
    timeout: Duration,
    logger: Arc<Logger>,
}

impl GoogleProvider {
    /// Create new Google Gemini provider
    pub fn new(
        api_key: String,
        default_model: String,
        logger: Arc<Logger>,
    ) -> Self {
        logger.log(
            LogLevel::Info,
            "google_provider",
            &format!("📡 Initializing Google Gemini provider with model: {}", default_model),
        );

        Self {
            api_key,
            default_model,
            embedding_model: "text-embedding-004".to_string(),
            timeout: Duration::from_secs(120),
            logger,
        }
    }

    /// Set embedding model
    #[allow(dead_code)]
    pub fn with_embedding_model(mut self, model: String) -> Self {
        self.embedding_model = model;
        self
    }

    /// Set timeout
    #[allow(dead_code)]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the API endpoint URL
    fn build_url(&self, model: &str, action: &str) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:{}?key={}",
            model, action, self.api_key
        )
    }
}

#[async_trait]
impl AIProvider for GoogleProvider {
    fn name(&self) -> &str {
        "Google"
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Network {
            requires_internet: true,
        }
    }

    async fn generate(
        &self,
        request: GenerationRequest,
    ) -> Result<GenerationResponse, ProviderError> {
        self.logger.log(
            LogLevel::Debug,
            "google_provider",
            &format!("🤖 [Google] Generating with model: {}", request.model),
        );

        let endpoint = self.build_url(&request.model, "generateContent");

        // Build system instruction if provided
        let system_instruction = request.system_prompt.as_ref().map(|s| {
            GeminiSystemInstruction {
                parts: vec![GeminiPart { text: s.clone() }],
            }
        });

        let gemini_request = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: request.prompt.clone(),
                }],
            }],
            system_instruction,
            generation_config: Some(GeminiGenerationConfig {
                temperature: Some(request.temperature),
                max_output_tokens: request.max_tokens,
            }),
        };

        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        let response = client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&gemini_request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            
            return Err(match status.as_u16() {
                401 | 403 => ProviderError::AuthenticationError(format!("[Google] {}", error_text)),
                429 => ProviderError::RateLimitError(format!("[Google] {}", error_text)),
                404 => ProviderError::ModelNotFound(format!("[Google] {}", error_text)),
                _ => ProviderError::NetworkError(format!("[Google] Status {}: {}", status, error_text)),
            });
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InternalError(format!("Failed to parse response: {}", e)))?;

        // Check for API error in response body
        if let Some(error) = gemini_response.error {
            return Err(ProviderError::InternalError(format!(
                "[Google] API error: {}",
                error.message
            )));
        }

        let text = gemini_response
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .and_then(|c| c.content.as_ref())
            .and_then(|c| c.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_default();

        let tokens_used = gemini_response
            .usage_metadata
            .and_then(|u| u.total_token_count)
            .unwrap_or(0);

        let finish_reason = gemini_response
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .and_then(|c| c.finish_reason.as_ref())
            .map(|r| match r.as_str() {
                "STOP" => FinishReason::Stop,
                "MAX_TOKENS" => FinishReason::Length,
                "SAFETY" => FinishReason::ContentFilter,
                _ => FinishReason::Stop,
            })
            .unwrap_or(FinishReason::Stop);

        let preview_len = std::cmp::min(text.len(), 100);
        let preview = &text[..preview_len];
        let suffix = if text.len() > 100 { "..." } else { "" };

        self.logger.log(
            LogLevel::Success,
            "google_provider",
            &format!("✅ [Google] Generated {} chars ({} tokens): '{}{}'", 
                text.len(), tokens_used, preview.replace('\n', " "), suffix),
        );

        Ok(GenerationResponse {
            text,
            model: request.model,
            tokens_used,
            finish_reason,
        })
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, ProviderError> {
        self.logger.log(
            LogLevel::Debug,
            "google_provider",
            &format!("🔢 [Google] Generating embedding for {} chars", text.len()),
        );

        let endpoint = self.build_url(&self.embedding_model, "embedContent");

        let embed_request = GeminiEmbedRequest {
            model: format!("models/{}", self.embedding_model),
            content: GeminiEmbedContent {
                parts: vec![GeminiPart {
                    text: text.to_string(),
                }],
            },
        };

        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        let response = client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&embed_request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 | 403 => ProviderError::AuthenticationError(error_text),
                429 => ProviderError::RateLimitError(error_text),
                _ => ProviderError::NetworkError(format!("Status {}: {}", status, error_text)),
            });
        }

        let embed_response: GeminiEmbedResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InternalError(format!("Failed to parse embedding response: {}", e)))?;

        if let Some(error) = embed_response.error {
            return Err(ProviderError::InternalError(error.message));
        }

        let embedding = embed_response
            .embedding
            .map(|e| e.values)
            .ok_or_else(|| ProviderError::InternalError("No embedding data returned".to_string()))?;

        self.logger.log(
            LogLevel::Success,
            "google_provider",
            &format!("✅ [Google] Generated embedding with {} dimensions", embedding.len()),
        );

        Ok(embedding)
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        self.logger.log(
            LogLevel::Debug,
            "google_provider",
            "📋 [Google] Listing available models",
        );

        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            self.api_key
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        let response = client
            .get(&endpoint)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::NetworkError(format!("Failed to list models: {}", error_text)));
        }

        let models_response: GeminiModelsResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InternalError(format!("Failed to parse models response: {}", e)))?;

        let models: Vec<ModelInfo> = models_response
            .models
            .unwrap_or_default()
            .into_iter()
            .filter(|m| {
                // Only include generateContent-capable models
                m.supported_generation_methods
                    .as_ref()
                    .map(|methods| methods.iter().any(|m| m == "generateContent"))
                    .unwrap_or(false)
            })
            .map(|m| {
                // Strip "models/" prefix from name
                let id = m.name.strip_prefix("models/").unwrap_or(&m.name).to_string();
                ModelInfo {
                    id: id.clone(),
                    name: m.display_name.unwrap_or(id),
                    context_length: m.input_token_limit.unwrap_or(32000),
                    supports_embeddings: false,
                    supports_function_calling: true,
                }
            })
            .collect();

        self.logger.log(
            LogLevel::Success,
            "google_provider",
            &format!("✅ [Google] Found {} generative models", models.len()),
        );

        Ok(models)
    }

    async fn health_check(&self) -> Result<ProviderHealth, ProviderError> {
        let start = Instant::now();

        match self.list_models().await {
            Ok(_) => Ok(ProviderHealth {
                healthy: true,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                error: None,
            }),
            Err(e) => Ok(ProviderHealth {
                healthy: false,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                error: Some(e.to_string()),
            }),
        }
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

    fn supports_embeddings(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn max_context_length(&self) -> usize {
        1000000 // Gemini 1.5 Pro has 1M context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_provider_creation() {
        let logger = Arc::new(Logger::new(false));
        let provider = GoogleProvider::new(
            "AIza-test".to_string(),
            "gemini-1.5-flash".to_string(),
            logger,
        );

        assert_eq!(provider.name(), "Google");
        assert!(provider.is_available());
        assert!(provider.supports_embeddings());
    }

    #[test]
    fn test_empty_api_key() {
        let logger = Arc::new(Logger::new(false));
        let provider = GoogleProvider::new(
            "".to_string(),
            "gemini-1.5-flash".to_string(),
            logger,
        );

        assert!(!provider.is_available());
    }

    #[test]
    fn test_url_building() {
        let logger = Arc::new(Logger::new(false));
        let provider = GoogleProvider::new(
            "test-key".to_string(),
            "gemini-1.5-flash".to_string(),
            logger,
        );

        let url = provider.build_url("gemini-1.5-flash", "generateContent");
        assert!(url.contains("gemini-1.5-flash"));
        assert!(url.contains("generateContent"));
        assert!(url.contains("key=test-key"));
    }
}

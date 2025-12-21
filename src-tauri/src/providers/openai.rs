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
// OpenAI Chat Completions API structures
// ============================================================================

#[derive(Debug, Clone, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Usage {
    #[serde(default)]
    prompt_tokens: usize,
    #[serde(default)]
    completion_tokens: usize,
    total_tokens: usize,
}

// ============================================================================
// Embeddings API structures
// ============================================================================

#[derive(Debug, Clone, Serialize)]
struct EmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Debug, Clone, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Clone, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

// ============================================================================
// Models API structures
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelData>,
}

#[derive(Debug, Clone, Deserialize)]
struct ModelData {
    id: String,
}

// ============================================================================
// OpenAI Provider Implementation
// ============================================================================

/// OpenAI-compatible provider (works with OpenAI, OpenRouter, Azure OpenAI, etc.)
pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
    default_model: String,
    embedding_model: String,
    timeout: Duration,
    logger: Arc<Logger>,
    provider_name: String, // "OpenAI", "OpenRouter", etc.
}

impl OpenAIProvider {
    /// Create new OpenAI provider
    pub fn new(
        api_key: String,
        default_model: String,
        logger: Arc<Logger>,
    ) -> Self {
        Self::with_base_url(
            api_key,
            "https://api.openai.com/v1".to_string(),
            default_model,
            "OpenAI".to_string(),
            logger,
        )
    }

    /// Create OpenRouter provider (OpenAI-compatible API)
    pub fn openrouter(
        api_key: String,
        default_model: String,
        logger: Arc<Logger>,
    ) -> Self {
        Self::with_base_url(
            api_key,
            "https://openrouter.ai/api/v1".to_string(),
            default_model,
            "OpenRouter".to_string(),
            logger,
        )
    }

    /// Create with custom base URL
    pub fn with_base_url(
        api_key: String,
        base_url: String,
        default_model: String,
        provider_name: String,
        logger: Arc<Logger>,
    ) -> Self {
        logger.log(
            LogLevel::Info,
            "openai_provider",
            &format!("📡 Initializing {} provider at {}", provider_name, base_url),
        );

        Self {
            api_key,
            base_url,
            default_model,
            embedding_model: "text-embedding-3-small".to_string(),
            timeout: Duration::from_secs(120),
            logger,
            provider_name,
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

    /// Check if this is OpenRouter (affects embedding support)
    fn is_openrouter(&self) -> bool {
        self.base_url.contains("openrouter.ai")
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    fn name(&self) -> &str {
        &self.provider_name
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
            "openai_provider",
            &format!("🤖 [{}] Generating with model: {}", self.provider_name, request.model),
        );

        let endpoint = format!("{}/chat/completions", self.base_url);

        // Build messages
        let mut messages = Vec::new();

        if let Some(system) = &request.system_prompt {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: system.clone(),
            });
        }

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
        });

        let chat_request = ChatCompletionRequest {
            model: request.model.clone(),
            messages,
            temperature: Some(request.temperature),
            max_tokens: request.max_tokens,
            stream: false,
        };

        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        let mut request_builder = client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&chat_request);

        // OpenRouter requires additional headers
        if self.is_openrouter() {
            request_builder = request_builder
                .header("HTTP-Referer", "https://github.com/m0nk111/TheCouncelOfDicks")
                .header("X-Title", "Council Of Dicks");
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            
            return Err(match status.as_u16() {
                401 => ProviderError::AuthenticationError(format!("[{}] {}", self.provider_name, error_text)),
                429 => ProviderError::RateLimitError(format!("[{}] {}", self.provider_name, error_text)),
                404 => ProviderError::ModelNotFound(format!("[{}] {}", self.provider_name, error_text)),
                _ => ProviderError::NetworkError(format!("[{}] Status {}: {}", self.provider_name, status, error_text)),
            });
        }

        let chat_response: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InternalError(format!("Failed to parse response: {}", e)))?;

        let text = chat_response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        let tokens_used = chat_response.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
        let input_tokens = chat_response.usage.as_ref().map(|u| u.prompt_tokens);
        let output_tokens = chat_response.usage.as_ref().map(|u| u.completion_tokens);

        let finish_reason = chat_response
            .choices
            .first()
            .and_then(|c| c.finish_reason.as_ref())
            .map(|r| match r.as_str() {
                "stop" => FinishReason::Stop,
                "length" => FinishReason::Length,
                "tool_calls" | "function_call" => FinishReason::ToolCalls,
                "content_filter" => FinishReason::ContentFilter,
                _ => FinishReason::Stop,
            })
            .unwrap_or(FinishReason::Stop);

        let preview_len = std::cmp::min(text.len(), 100);
        let preview = &text[..preview_len];
        let suffix = if text.len() > 100 { "..." } else { "" };

        self.logger.log(
            LogLevel::Success,
            "openai_provider",
            &format!("✅ [{}] Generated {} chars ({} tokens): '{}{}'", 
                self.provider_name, text.len(), tokens_used, preview.replace('\n', " "), suffix),
        );

        Ok(GenerationResponse {
            text,
            model: request.model,
            tokens_used,
            input_tokens,
            output_tokens,
            finish_reason,
        })
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, ProviderError> {
        // OpenRouter doesn't support embeddings
        if self.is_openrouter() {
            return Err(ProviderError::NotSupported(
                "OpenRouter does not support embeddings - use Ollama or OpenAI directly".to_string(),
            ));
        }

        self.logger.log(
            LogLevel::Debug,
            "openai_provider",
            &format!("🔢 [{}] Generating embedding for {} chars", self.provider_name, text.len()),
        );

        let endpoint = format!("{}/embeddings", self.base_url);

        let embed_request = EmbeddingRequest {
            model: self.embedding_model.clone(),
            input: text.to_string(),
        };

        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        let response = client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&embed_request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 => ProviderError::AuthenticationError(error_text),
                429 => ProviderError::RateLimitError(error_text),
                _ => ProviderError::NetworkError(format!("Status {}: {}", status, error_text)),
            });
        }

        let embed_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InternalError(format!("Failed to parse embedding response: {}", e)))?;

        let embedding = embed_response
            .data
            .first()
            .map(|d| d.embedding.clone())
            .ok_or_else(|| ProviderError::InternalError("No embedding data returned".to_string()))?;

        self.logger.log(
            LogLevel::Success,
            "openai_provider",
            &format!("✅ [{}] Generated embedding with {} dimensions", self.provider_name, embedding.len()),
        );

        Ok(embedding)
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        self.logger.log(
            LogLevel::Debug,
            "openai_provider",
            &format!("📋 [{}] Listing available models", self.provider_name),
        );

        let endpoint = format!("{}/models", self.base_url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        let response = client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::NetworkError(format!("Failed to list models: {}", error_text)));
        }

        let models_response: ModelsResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InternalError(format!("Failed to parse models response: {}", e)))?;

        let models: Vec<ModelInfo> = models_response
            .data
            .into_iter()
            .map(|m| ModelInfo {
                id: m.id.clone(),
                name: m.id,
                context_length: 128000, // Default, varies by model
                supports_embeddings: false,
                supports_function_calling: true,
            })
            .collect();

        self.logger.log(
            LogLevel::Success,
            "openai_provider",
            &format!("✅ [{}] Found {} models", self.provider_name, models.len()),
        );

        Ok(models)
    }

    async fn health_check(&self) -> Result<ProviderHealth, ProviderError> {
        let start = Instant::now();

        // Try listing models as health check
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
        // OpenRouter doesn't support embeddings
        !self.is_openrouter()
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn max_context_length(&self) -> usize {
        128000 // GPT-4 Turbo default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let logger = Arc::new(Logger::new(false));
        let provider = OpenAIProvider::new(
            "sk-test".to_string(),
            "gpt-4".to_string(),
            logger,
        );

        assert_eq!(provider.name(), "OpenAI");
        assert!(provider.is_available());
        assert!(provider.supports_embeddings());
    }

    #[test]
    fn test_openrouter_provider_creation() {
        let logger = Arc::new(Logger::new(false));
        let provider = OpenAIProvider::openrouter(
            "sk-or-test".to_string(),
            "openai/gpt-4".to_string(),
            logger,
        );

        assert_eq!(provider.name(), "OpenRouter");
        assert!(provider.is_available());
        assert!(!provider.supports_embeddings()); // OpenRouter doesn't support embeddings
    }

    #[test]
    fn test_empty_api_key() {
        let logger = Arc::new(Logger::new(false));
        let provider = OpenAIProvider::new(
            "".to_string(),
            "gpt-4".to_string(),
            logger,
        );

        assert!(!provider.is_available());
    }
}

pub mod config;
pub mod google;
pub mod ollama;
pub mod openai;
pub mod registry;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

pub use google::GoogleProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;

/// AI Provider error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderError {
    NetworkError(String),
    AuthenticationError(String),
    RateLimitError(String),
    ModelNotFound(String),
    InvalidRequest(String),
    NotSupported(String),
    InternalError(String),
    AllProvidersFailed,
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ProviderError::AuthenticationError(msg) => write!(f, "Authentication failed: {}", msg),
            ProviderError::RateLimitError(msg) => write!(f, "Rate limit exceeded: {}", msg),
            ProviderError::ModelNotFound(msg) => write!(f, "Model not found: {}", msg),
            ProviderError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            ProviderError::NotSupported(msg) => write!(f, "Not supported: {}", msg),
            ProviderError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            ProviderError::AllProvidersFailed => write!(f, "All providers failed"),
        }
    }
}

impl std::error::Error for ProviderError {}

/// Provider type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub enum ProviderType {
    Network { requires_internet: bool },
    Local { bundled: bool },
    Hybrid,
}

/// Text generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub model: String,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub temperature: f32,
    pub max_tokens: Option<usize>,
    pub stream: bool,
}

/// Text generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    pub text: String,
    pub model: String,
    pub tokens_used: usize,
    /// Input/prompt tokens (if available from provider)
    #[serde(default)]
    pub input_tokens: Option<usize>,
    /// Output/completion tokens (if available from provider)
    #[serde(default)]
    pub output_tokens: Option<usize>,
    pub finish_reason: FinishReason,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_length: usize,
    pub supports_embeddings: bool,
    pub supports_function_calling: bool,
}

/// Provider health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub healthy: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

/// AI Provider trait - implemented by all providers
#[async_trait]
#[allow(dead_code)]
pub trait AIProvider: Send + Sync {
    /// Provider identification
    fn name(&self) -> &str;
    fn provider_type(&self) -> ProviderType;

    /// Core capabilities
    async fn generate(
        &self,
        request: GenerationRequest,
    ) -> Result<GenerationResponse, ProviderError>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ProviderError>;
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError>;

    /// Health & status
    async fn health_check(&self) -> Result<ProviderHealth, ProviderError>;
    fn is_available(&self) -> bool;

    /// Configuration
    fn supports_embeddings(&self) -> bool;
    fn supports_streaming(&self) -> bool;
    fn max_context_length(&self) -> usize;
}

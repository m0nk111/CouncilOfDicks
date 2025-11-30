// Simple HTTP server for Council Of Dicks - MVP version
// Web-first architecture: browser access + optional native app

use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

// ========================================
// Configuration
// ========================================

#[derive(Debug, Clone)]
pub struct HttpServerConfig {
    pub port: u16,
    pub host: String,
    pub enable_cors: bool,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            enable_cors: true,
        }
    }
}

// ========================================
// Server
// ========================================

pub struct HttpServer {
    config: HttpServerConfig,
    state: Arc<AppState>,
}

impl HttpServer {
    pub fn new(config: HttpServerConfig, state: Arc<AppState>) -> Self {
        Self { config, state }
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let router = self.build_router();
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        
        println!("✅ HTTP server listening on {}", addr);
        
        axum::serve(listener, router).await?;
        Ok(())
    }

    fn build_router(self) -> Router {
        let mut router = Router::new()
            // Health check
            .route("/health", get(health_check))
            
            // Ollama API
            .route("/api/ollama/ask", post(ollama_ask))
            
            // Config API
            .route("/api/config", get(config_get))
            
            // Static files (for web UI)
            .fallback(static_handler);

        // Enable CORS for browser access
        if self.config.enable_cors {
            router = router.layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any)
            );
        }

        router.with_state(self.state)
    }
}

// ========================================
// Error Handling
// ========================================

#[derive(Debug)]
enum ApiError {
    BadRequest(String),
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        
        (status, Json(serde_json::json!({
            "error": message
        }))).into_response()
    }
}

// ========================================
// Request/Response Types
// ========================================

#[derive(Debug, Deserialize)]
struct OllamaAskRequest {
    model: Option<String>,
    prompt: String,
}

#[derive(Debug, Serialize)]
struct OllamaAskResponse {
    response: String,
}

#[derive(Debug, Serialize)]
struct ConfigResponse {
    ollama_url: String,
    ollama_model: String,
    debug_enabled: bool,
}

// ========================================
// Handlers
// ========================================

async fn health_check() -> &'static str {
    "OK"
}

async fn ollama_ask(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OllamaAskRequest>,
) -> Result<Json<OllamaAskResponse>, ApiError> {
    let config = state.get_config();
    let model = req.model.unwrap_or(config.ollama_model.clone());
    
    let response = crate::ollama::ask_ollama_internal(&state, model, req.prompt)
        .await
        .map_err(|e| ApiError::InternalError(e))?;
    
    Ok(Json(OllamaAskResponse { response }))
}

async fn config_get(
    State(state): State<Arc<AppState>>,
) -> Json<ConfigResponse> {
    let config = state.get_config();
    Json(ConfigResponse {
        ollama_url: config.ollama_url.clone(),
        ollama_model: config.ollama_model.clone(),
        debug_enabled: config.debug_enabled,
    })
}

async fn static_handler() -> impl IntoResponse {
    // TODO: Serve frontend files from dist/
    // For now, return a simple HTML page
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html")],
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Council Of Dicks</title>
    <meta charset="utf-8">
    <style>
        body { font-family: monospace; max-width: 800px; margin: 50px auto; padding: 20px; }
        h1 { border-bottom: 2px solid #333; }
        code { background: #f4f4f4; padding: 2px 6px; }
        pre { background: #f4f4f4; padding: 15px; overflow-x: auto; }
    </style>
</head>
<body>
    <h1>🏛️ Council Of Dicks - HTTP API</h1>
    <p>Web-first architecture is running!</p>
    
    <h2>Available Endpoints:</h2>
    <ul>
        <li><code>GET /health</code> - Health check</li>
        <li><code>GET /api/config</code> - Get configuration</li>
        <li><code>POST /api/ollama/ask</code> - Ask Ollama a question</li>
    </ul>
    
    <h2>Example: Ask Ollama</h2>
    <pre>curl -X POST http://localhost:8080/api/ollama/ask \
  -H "Content-Type: application/json" \
  -d '{"prompt": "What is the meaning of life?"}'</pre>
    
    <p><em>Frontend UI coming soon...</em></p>
</body>
</html>"#
    )
}

// ========================================
// Tests
// ========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_server_config_default() {
        let config = HttpServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "127.0.0.1");
        assert!(config.enable_cors);
    }
}

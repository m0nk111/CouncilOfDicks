// Simple HTTP server for Council Of Dicks - MVP version
// Web-first architecture: browser access + optional native app

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
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
        println!("🔨 Building router with chat endpoints...");
        let mut router = Router::new()
            // Health check
            .route("/health", get(health_check))
            // Ollama API
            .route("/api/ollama/ask", post(ollama_ask))
            // Config API
            .route("/api/config", get(config_get).post(config_save))
            .route("/api/user/handle", post(user_handle_set))
            .route("/api/constitution", get(constitution_get))
            // Council API
            .route("/api/council/generate_question", post(generate_question))
            .route("/api/council/session", post(council_session_get))
            .route("/api/council/sessions", get(council_sessions_list))
            // PoHV API
            .route("/api/pohv/status", get(pohv_status))
            // Agent API
            .route("/api/agents", get(agent_list))
            .route("/api/agents/get", post(agent_get))
            .route("/api/agents/update", post(agent_update))
            .route("/api/agents/reset-identity", post(agent_reset_identity))
            // Chat API
            .route("/api/chat/message", post(chat_message_send))
            .route("/api/chat/messages", post(chat_messages_get))
            // WebSocket for real-time chat
            .route("/ws/chat", get(websocket_handler))
            // Static files (for web UI)
            .fallback(static_handler);

        // Enable CORS for browser access
        if self.config.enable_cors {
            router = router.layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
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
    #[allow(dead_code)]
    BadRequest(String),
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (
            status,
            Json(serde_json::json!({
                "error": message
            })),
        )
            .into_response()
    }
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(msg: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.to_string()),
        }
    }
}

struct AppError(String);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": self.0
            })),
        )
            .into_response()
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

#[derive(Debug, Clone, Serialize, Deserialize)]
// #[allow(dead_code)]
struct ConfigResponse {
    ollama_url: String,
    ollama_model: String,
    debug_enabled: bool,
    user_handle: String,
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

    let response = crate::ollama::ask_ollama_internal(&state, model, req.prompt, None)
        .await
        .map_err(ApiError::InternalError)?;

    Ok(Json(OllamaAskResponse { response }))
}

async fn config_get(State(state): State<Arc<AppState>>) -> Json<ApiResponse<crate::config::AppConfig>> {
    Json(ApiResponse::success(state.get_config()))
}

#[derive(Deserialize)]
struct ConfigSaveRequest {
    config: crate::config::AppConfig,
}

async fn config_save(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ConfigSaveRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    state.update_config(|c| {
        *c = payload.config.clone();
    });
    
    // Persist to disk
    payload.config.save().map_err(AppError)?;
    
    state.log_info("http_server", "Configuration saved via HTTP API");
    Ok(Json(ApiResponse::success(())))
}

#[derive(Deserialize)]
struct UserHandleRequest {
    handle: String,
}

async fn user_handle_set(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserHandleRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    state.update_config(|c| {
        c.user_handle = payload.handle.clone();
    });
    
    // Persist to disk
    let config = state.get_config();
    config.save().map_err(AppError)?;
    
    state.log_info("http_server", &format!("User handle updated to: {}", payload.handle));
    Ok(Json(ApiResponse::success(())))
}

async fn constitution_get(State(state): State<Arc<AppState>>) -> Json<ApiResponse<String>> {
    let content = state.constitution_manager.get_content();
    Json(ApiResponse::success(content))
}

async fn generate_question(
    State(state): State<Arc<AppState>>,
) -> Result<Json<String>, ApiError> {
    let config = state.get_config();
    let model = config.ollama_model.clone();
    let url = config.ollama_url.clone();

    let prompt = "Generate a single, short, provocative, and open-ended philosophical or ethical question for an AI council to debate. The question should be deep and require nuanced thinking. Do not include any preamble, explanation, or quotes. Just the question itself.".to_string();

    match crate::ollama::ask_ollama(&url, &model, prompt).await {
        Ok(question) => Ok(Json(question.trim().to_string())),
        Err(e) => Err(ApiError::InternalError(format!("Failed to generate question: {}", e))),
    }
}

#[derive(Deserialize)]
struct CouncilSessionRequest {
    #[serde(rename = "sessionId")]
    session_id: String,
}

async fn council_session_get(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CouncilSessionRequest>,
) -> Result<Json<ApiResponse<crate::protocol::CouncilSession>>, ApiError> {
    let session = state.council_manager.get_session(&payload.session_id)
        .await
        .ok_or_else(|| ApiError::BadRequest("Session not found".to_string()))?;
    Ok(Json(ApiResponse::success(session)))
}

#[derive(Serialize)]
struct CouncilSessionsListResponse {
    sessions: Vec<crate::protocol::CouncilSession>,
}

async fn council_sessions_list(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<CouncilSessionsListResponse>> {
    let sessions = state.council_manager.list_sessions().await;
    Json(ApiResponse::success(CouncilSessionsListResponse { sessions }))
}

async fn pohv_status(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<crate::pohv::PoHVState>> {
    let status = state.pohv_system.get_state();
    Json(ApiResponse::success(status))
}

async fn agent_list(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<crate::agents::Agent>>> {
    let agents = state.agent_pool.list_agents().await;
    Json(ApiResponse::success(agents))
}

#[derive(Deserialize)]
struct AgentGetRequest {
    agent_id: String,
}

async fn agent_get(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AgentGetRequest>,
) -> Result<Json<ApiResponse<crate::agents::Agent>>, ApiError> {
    let agent = state.agent_pool.get_agent(&payload.agent_id).await
        .map_err(ApiError::InternalError)?;
    Ok(Json(ApiResponse::success(agent)))
}

async fn agent_update(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<crate::agents::Agent>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    state.agent_pool.update_agent(payload).await
        .map_err(ApiError::InternalError)?;
    Ok(Json(ApiResponse::success(())))
}

#[derive(Deserialize)]
struct AgentResetIdentityRequest {
    agent_id: String,
    user_hint: Option<String>,
}

#[derive(Serialize)]
struct AgentResetIdentityResponse {
    agent: crate::agents::Agent,
    identity: crate::providers::config::AgentIdentity,
}

async fn agent_reset_identity(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AgentResetIdentityRequest>,
) -> Result<Json<ApiResponse<AgentResetIdentityResponse>>, ApiError> {
    state.log_info("http_server", &format!("🎭 Resetting identity for agent: {}", payload.agent_id));
    
    // Get existing agent
    let existing_agent = state.agent_pool.get_agent(&payload.agent_id).await
        .map_err(ApiError::InternalError)?;
    
    // Get existing agent names to avoid duplicates
    let existing_names: Vec<String> = state.agent_pool
        .list_agents()
        .await
        .iter()
        .filter(|a| a.id != payload.agent_id)
        .map(|a| a.name.clone())
        .collect();
    
    // Generate new identity
    let identity = crate::providers::config::generate_agent_identity(
        &existing_agent.model,
        &existing_agent.provider,
        &existing_names,
        payload.user_hint.as_deref(),
    ).await.map_err(ApiError::InternalError)?;
    
    // Generate new system prompt
    let system_prompt = format!(
        "You are {}, a council member with the role of {}. {}\n\nYour job is to participate in council deliberations, bringing your unique perspective as a {}. Stay in character and provide thoughtful, substantive contributions to discussions.",
        identity.name, identity.role, identity.tagline, identity.role
    );
    
    // Update agent with new identity
    let mut updated_agent = existing_agent;
    updated_agent.name = identity.name.clone();
    updated_agent.handle = identity.handle.clone();
    updated_agent.system_prompt = system_prompt;
    updated_agent.metadata.insert("role".to_string(), identity.role.clone());
    
    state.agent_pool.update_agent(updated_agent.clone()).await
        .map_err(ApiError::InternalError)?;
    
    state.log_success("http_server", &format!(
        "🎭 Agent {} is now: {} (@{}) - {}",
        payload.agent_id, identity.name, identity.handle, identity.role
    ));
    
    Ok(Json(ApiResponse::success(AgentResetIdentityResponse {
        agent: updated_agent,
        identity,
    })))
}

#[derive(Deserialize)]
struct ChatMessagePayload {
    content: String,
    channel: Option<crate::chat::ChannelType>,
}

async fn chat_message_send(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChatMessagePayload>,
) -> impl IntoResponse {
    state.log_debug("http_server", &format!("📩 Received chat message: '{}' for channel {:?}", payload.content, payload.channel));

    let msg = crate::chat::Message {
        id: uuid::Uuid::new_v4().to_string(),
        channel: payload.channel.unwrap_or(crate::chat::ChannelType::General),
        author: state.get_config().user_handle,
        author_type: crate::chat::AuthorType::Human,
        content: payload.content,
        timestamp: chrono::Utc::now(),
        signature: None,
        reply_to: None,
        reactions: vec![],
    };

    // Add to channel manager
    if let Err(e) = state.channel_manager.send_message(msg.clone()) {
        return Json(ApiResponse::error(&format!("Failed to send message: {}", e)));
    }

    // Broadcast to WS
    let _ = state.websocket_broadcast.send(msg.clone());
    
    Json(ApiResponse::success(msg))
}

#[derive(Deserialize)]
struct ChatMessagesRequest {
    channel: crate::chat::ChannelType,
    limit: usize,
    offset: usize,
}

async fn chat_messages_get(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChatMessagesRequest>,
) -> Result<Json<ApiResponse<Vec<crate::chat::Message>>>, ApiError> {
    let messages = state.channel_manager.get_messages(payload.channel, payload.limit, payload.offset)
        .map_err(ApiError::InternalError)?;
    Ok(Json(ApiResponse::success(messages)))
}

async fn static_handler() -> impl IntoResponse {
    // Serve test page from workspace root
    // TODO: Serve production frontend from dist/ when built
    let test_page_path = std::path::PathBuf::from("./test-web-mode.html");

    if test_page_path.exists() {
        match tokio::fs::read_to_string(test_page_path).await {
            Ok(content) => {
                return (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "text/html")],
                    content,
                )
                    .into_response();
            }
            Err(_) => {
                // Fall through to default page
            }
        }
    }

    // Fallback: simple API documentation page
    let fallback_html = r#"<!DOCTYPE html>
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
</html>"#;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html")],
        fallback_html,
    )
        .into_response()
}

// ========================================
// WebSocket Handler
// ========================================

/// WebSocket upgrade handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket_connection(socket, state))
}

/// Handle individual WebSocket connection
async fn websocket_connection(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.websocket_broadcast.subscribe();

    // Send welcome message
    let welcome = serde_json::json!({
        "type": "welcome",
        "message": "Connected to Council Of Dicks chat"
    });

    if socket
        .send(Message::Text(welcome.to_string()))
        .await
        .is_err()
    {
        return;
    }

    // Forward broadcast messages to this client
    let mut send_socket = socket;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                // Receive messages from broadcast channel
                msg = rx.recv() => {
                    match msg {
                        Ok(chat_msg) => {
                            // Serialize and send to client
                            let json = match serde_json::to_string(&chat_msg) {
                                Ok(j) => j,
                                Err(e) => {
                                    eprintln!("❌ Failed to serialize message: {}", e);
                                    continue;
                                }
                            };

                            if send_socket.send(Message::Text(json)).await.is_err() {
                                // Client disconnected
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            eprintln!("⚠️ WebSocket client lagged, skipped {} messages", skipped);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }

                // Receive messages from client (for future bidirectional chat)
                result = send_socket.recv() => {
                    match result {
                        Some(Ok(Message::Text(_text))) => {
                            // TODO: Handle incoming chat messages from client
                            // For now, WebSocket is receive-only (server->client)
                        }
                        Some(Ok(Message::Close(_))) | None => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        println!("🔌 WebSocket client disconnected");
    });
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

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    agents::AgentPool,
    council::CouncilSessionManager,
    AppState,
};

#[derive(Clone)]
pub struct WebState {
    pub app_state: Arc<AppState>,
    pub council_manager: Arc<CouncilSessionManager>,
    pub agent_pool: Arc<AgentPool>,
}

// Request/Response types matching Tauri commands
#[derive(Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub model_name: String,
    pub system_prompt: Option<String>,
    pub tools: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct CouncilSessionRequest {
    pub question: String,
    pub agent_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn err(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

// Health check endpoint
async fn health_check() -> (StatusCode, Json<ApiResponse<String>>) {
    (StatusCode::OK, Json(ApiResponse::ok("OK".to_string())))
}

// Get app config
async fn get_config(State(state): State<WebState>) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let config = state.app_state.config.lock().unwrap();
    (StatusCode::OK, Json(ApiResponse::ok(serde_json::json!({
        "ollama_url": config.ollama_url,
        "ollama_model": config.ollama_model,
        "debug_enabled": config.debug_enabled,
    }))))
}

// Agent endpoints
async fn list_agents(State(state): State<WebState>) -> (StatusCode, Json<ApiResponse<Vec<crate::agents::Agent>>>) {
    let agents = state.agent_pool.list_agents().await;
    (StatusCode::OK, Json(ApiResponse::ok(agents)))
}

async fn create_agent(
    State(state): State<WebState>,
    Json(req): Json<CreateAgentRequest>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    use crate::agents::Agent;
    
    let mut agent = Agent::new(
        req.name,
        req.model_name,
        req.system_prompt.unwrap_or_default(),
    );
    
    if let Some(tools) = req.tools {
        agent.tools = tools;
    }
    
    match state.agent_pool.add_agent(agent).await {
        Ok(agent_id) => (StatusCode::OK, Json(ApiResponse::ok(agent_id))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<String>::err(e))),
    }
}

async fn delete_agent(
    State(state): State<WebState>,
    Json(agent_id): Json<String>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    match state.agent_pool.remove_agent(&agent_id).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("Agent deleted".to_string()))),
        Err(e) => (StatusCode::NOT_FOUND, Json(ApiResponse::<String>::err(e))),
    }
}

// Council endpoints
async fn list_council_sessions(State(state): State<WebState>) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let sessions = state.council_manager.list_sessions().await;
    (StatusCode::OK, Json(ApiResponse::ok(serde_json::json!({
        "sessions": sessions
    }))))
}

async fn get_council_session(
    State(state): State<WebState>,
    Json(session_id): Json<String>,
) -> (StatusCode, Json<ApiResponse<Option<crate::council::CouncilSession>>>) {
    let session = state.council_manager.get_session(&session_id).await;
    (StatusCode::OK, Json(ApiResponse::ok(session)))
}

async fn create_council_session(
    State(state): State<WebState>,
    Json(req): Json<CouncilSessionRequest>,
) -> impl IntoResponse {
    let config = state.app_state.config.lock().unwrap();
    let ollama_url = config.ollama_url.clone();
    drop(config);

    let result = state.council_manager.create_session_with_agents(
        req.question,
        state.agent_pool.clone(),
        req.agent_ids,
        &ollama_url,
    ).await;
    
    match result {
        Ok(session_id) => (StatusCode::OK, Json(ApiResponse::ok(session_id))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<String>::err(e))),
    }
}

// Build the web server router
pub fn create_router(state: WebState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_check))
        .route("/api/config", get(get_config))
        .route("/api/agents", get(list_agents))
        .route("/api/agents/create", post(create_agent))
        .route("/api/agents/delete", post(delete_agent))
        .route("/api/council/sessions", get(list_council_sessions))
        .route("/api/council/session", post(get_council_session))
        .route("/api/council/create", post(create_council_session))
        .layer(cors)
        .with_state(state)
}

pub async fn start_web_server(
    app_state: Arc<AppState>,
    council_manager: Arc<CouncilSessionManager>,
    agent_pool: Arc<AgentPool>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = WebState {
        app_state,
        council_manager,
        agent_pool,
    };

    let app = create_router(state);
    let addr = format!("0.0.0.0:{}", port);
    
    println!("🌐 Web server starting on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
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
async fn health_check() -> Response {
    (StatusCode::OK, Json(ApiResponse::ok("OK".to_string()))).into_response()
}

// Get app config
async fn get_config(State(state): State<WebState>) -> Response {
    let config = state.app_state.config.lock().unwrap();
    (StatusCode::OK, Json(ApiResponse::ok(serde_json::json!({
        "ollama_url": config.ollama_url,
        "ollama_model": config.ollama_model,
        "debug_enabled": config.debug_enabled,
    })))).into_response()
}

// Agent endpoints
async fn list_agents(State(state): State<WebState>) -> Response {
    let agents = state.agent_pool.list_agents().await;
    (StatusCode::OK, Json(ApiResponse::ok(agents))).into_response()
}

async fn create_agent(
    State(state): State<WebState>,
    Json(req): Json<CreateAgentRequest>,
) -> Response {
    use crate::agents::Agent;
    
    let mut agent = Agent::new(
        req.name,
        req.model_name,
        req.system_prompt.unwrap_or_default(),
    );
    
    if let Some(tools) = req.tools {
        agent.enabled_tools = tools;
    }
    
    match state.agent_pool.add_agent(agent).await {
        Ok(agent_id) => (StatusCode::OK, Json(ApiResponse::ok(agent_id))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<String>::err(e))).into_response(),
    }
}

async fn delete_agent(
    State(state): State<WebState>,
    Json(agent_id): Json<String>,
) -> Response {
    match state.agent_pool.remove_agent(&agent_id).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("Agent deleted".to_string()))).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(ApiResponse::<String>::err(e))).into_response(),
    }
}

// Council endpoints
async fn list_council_sessions(State(state): State<WebState>) -> Response {
    let sessions = state.council_manager.list_sessions().await;
    (StatusCode::OK, Json(ApiResponse::ok(serde_json::json!({
        "sessions": sessions
    })))).into_response()
}

async fn get_council_session(
    State(state): State<WebState>,
    Json(session_id): Json<String>,
) -> Response {
    let session = state.council_manager.get_session(&session_id).await;
    (StatusCode::OK, Json(ApiResponse::ok(session))).into_response()
}

async fn create_council_session(
    State(state): State<WebState>,
    Json(req): Json<CouncilSessionRequest>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<String>>)> {
    let config = state.app_state.config.lock().expect("Failed to lock config");
    let ollama_url = config.ollama_url.clone();
    drop(config);

    match state.council_manager.create_session_with_agents(
        req.question,
        state.agent_pool.clone(),
        req.agent_ids,
        &ollama_url,
    ).await {
        Ok(session_id) => Ok(Json(ApiResponse::ok(session_id))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e)))),
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
        // .route("/api/council/create", post(create_council_session)) // TODO: Fix Handler trait issue
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

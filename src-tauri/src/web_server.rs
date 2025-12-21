use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        Json, Query, State,
    },
    http::{StatusCode, Method, header},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    agents::AgentPool,
    chat::{AuthorType, ChannelType, Message},
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

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub channel: String,
    pub author: String,
    pub author_type: String,
    pub content: String,
    pub signature: Option<String>,
}

#[derive(Deserialize)]
pub struct GetMessagesRequest {
    pub channel: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Deserialize)]
pub struct TopicSetRequest {
    pub topic: String,
    pub interval: u64,
}

#[derive(Deserialize)]
pub struct TopicHistoryRequest {
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct SetHandleRequest {
    pub handle: String,
}

#[derive(Deserialize)]
pub struct AgentResetIdentityRequest {
    pub agent_id: String,
    pub user_hint: Option<String>,
}

#[derive(Serialize)]
pub struct AgentResetIdentityResponse {
    pub agent: crate::agents::Agent,
    pub identity: crate::providers::config::AgentIdentity,
}

#[derive(Serialize)]
pub struct CouncilSessionsListResponse {
    pub sessions: Vec<crate::protocol::CouncilSession>,
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
    (
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({
            "ollama_url": config.ollama_url,
            "ollama_model": config.ollama_model,
            "debug_enabled": config.debug_enabled,
            "user_handle": config.user_handle,
        }))),
    )
        .into_response()
}

// Agent endpoints

/// Agent with stats included
#[derive(Serialize)]
struct AgentWithStats {
    #[serde(flatten)]
    agent: crate::agents::Agent,
    stats: crate::agents::AgentStats,
}

async fn list_agents(State(state): State<WebState>) -> Response {
    let agents = state.agent_pool.list_agents().await;
    let all_stats = state.agent_pool.get_all_stats().await;
    
    let agents_with_stats: Vec<AgentWithStats> = agents.into_iter().map(|agent| {
        let stats = all_stats.get(&agent.id).cloned().unwrap_or_default();
        AgentWithStats { agent, stats }
    }).collect();
    
    (StatusCode::OK, Json(ApiResponse::ok(agents_with_stats))).into_response()
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

async fn delete_agent(State(state): State<WebState>, Json(payload): Json<Value>) -> Response {
    let agent_id = match payload {
        Value::String(id) => Some(id),
        Value::Object(map) => map
            .get("agent_id")
            .or_else(|| map.get("id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        _ => None,
    };

    let Some(agent_id) = agent_id else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::err(
                "Missing agent_id in delete request".to_string(),
            )),
        )
            .into_response();
    };

    match state.agent_pool.remove_agent(&agent_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::ok("Agent deleted".to_string())),
        )
            .into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(ApiResponse::<String>::err(e))).into_response(),
    }
}

async fn get_all_agent_stats(State(state): State<WebState>) -> Response {
    let stats = state.agent_pool.get_all_stats().await;
    (StatusCode::OK, Json(ApiResponse::ok(stats))).into_response()
}

async fn get_agent_stats(
    State(state): State<WebState>,
    axum::extract::Path(agent_id): axum::extract::Path<String>,
) -> Response {
    let stats = state.agent_pool.get_agent_stats(&agent_id).await;
    (StatusCode::OK, Json(ApiResponse::ok(stats))).into_response()
}

async fn reset_agent_identity(
    State(state): State<WebState>,
    Json(payload): Json<AgentResetIdentityRequest>,
) -> Response {
    state.app_state.log_info("web_server", &format!("🎭 Resetting identity for agent: {}", payload.agent_id));
    
    // Get existing agent
    let existing_agent = match state.agent_pool.get_agent(&payload.agent_id).await {
        Ok(agent) => agent,
        Err(e) => return (StatusCode::NOT_FOUND, Json(ApiResponse::<String>::err(format!("Agent not found: {}", e)))).into_response(),
    };
    
    // Get existing agent names to avoid duplicates
    let existing_names: Vec<String> = state.agent_pool
        .list_agents()
        .await
        .iter()
        .filter(|a| a.id != payload.agent_id)
        .map(|a| a.name.clone())
        .collect();
    
    // Generate new identity (use agent's custom timeout if set)
    let identity = match crate::providers::config::generate_agent_identity(
        &existing_agent.model,
        &existing_agent.provider,
        &existing_names,
        payload.user_hint.as_deref(),
        existing_agent.timeout_secs,
    ).await {
        Ok(id) => id,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<String>::err(format!("Failed to generate identity: {}", e)))).into_response(),
    };
    
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
    
    if let Err(e) = state.agent_pool.update_agent(updated_agent.clone()).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<String>::err(format!("Failed to update agent: {}", e)))).into_response();
    }
    
    state.app_state.log_success("web_server", &format!(
        "🎭 Agent {} is now: {} (@{}) - {}",
        payload.agent_id, identity.name, identity.handle, identity.role
    ));
    
    (StatusCode::OK, Json(ApiResponse::ok(AgentResetIdentityResponse {
        agent: updated_agent,
        identity,
    }))).into_response()
}

// Council endpoints
async fn list_council_sessions(State(state): State<WebState>) -> Response {
    let sessions = state.council_manager.list_sessions().await;
    (StatusCode::OK, Json(ApiResponse::ok(CouncilSessionsListResponse { sessions }))).into_response()
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
) -> Response {
    let (ollama_url, auth) = {
        let config = state
            .app_state
            .config
            .lock()
            .expect("Failed to lock config");
        // Ollama Guardian uses username-only auth (app name), password is optional
        let auth = config.ollama_username.as_ref().map(|u| {
            (u.clone(), config.ollama_password.clone().unwrap_or_default())
        });
        (config.ollama_url.clone(), auth)
    };

    match state
        .council_manager
        .create_session_with_agents(
            req.question,
            state.agent_pool.clone(),
            req.agent_ids,
            &ollama_url,
            auth,
        )
        .await
    {
        Ok(session_id) => (StatusCode::OK, Json(ApiResponse::ok(session_id))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<String>::err(e)),
        )
            .into_response(),
    }
}

// Chat endpoints
async fn send_message(
    State(state): State<WebState>,
    Json(req): Json<SendMessageRequest>,
) -> Response {
    let channel_type = match ChannelType::from_str(&req.channel) {
        Some(ct) => ct,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<String>::err(format!(
                    "Invalid channel: {}",
                    req.channel
                ))),
            )
                .into_response()
        }
    };

    let author_type = match req.author_type.as_str() {
        "human" => AuthorType::Human,
        "ai" => AuthorType::AI,
        "system" => AuthorType::System,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<String>::err(format!(
                    "Invalid author type: {}",
                    req.author_type
                ))),
            )
                .into_response()
        }
    };

    let mut message = Message::new(channel_type, req.author, author_type, req.content);

    if let Some(sig) = req.signature {
        message = message.with_signature(sig);
    }

    match state
        .app_state
        .channel_manager
        .send_message(message.clone())
    {
        Ok(msg_id) => {
            // Broadcast to WebSocket clients if available
            let _ = state.app_state.websocket_broadcast.send(message);
            (StatusCode::OK, Json(ApiResponse::ok(msg_id))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<String>::err(e)),
        )
            .into_response(),
    }
}

async fn get_messages(
    State(state): State<WebState>,
    Json(req): Json<GetMessagesRequest>,
) -> Response {
    let channel_type = match ChannelType::from_str(&req.channel) {
        Some(ct) => ct,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<Vec<Message>>::err(format!(
                    "Invalid channel: {}",
                    req.channel
                ))),
            )
                .into_response()
        }
    };

    match state.app_state.channel_manager.get_messages(
        channel_type,
        req.limit.unwrap_or(50),
        req.offset.unwrap_or(0),
    ) {
        Ok(messages) => (StatusCode::OK, Json(ApiResponse::ok(messages))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<Vec<Message>>::err(e)),
        )
            .into_response(),
    }
}

// PoHV endpoints
async fn get_pohv_status(State(state): State<WebState>) -> Response {
    let status = state.app_state.pohv_system.get_state();
    (StatusCode::OK, Json(ApiResponse::ok(status))).into_response()
}

async fn pohv_heartbeat(State(state): State<WebState>) -> Response {
    state.app_state.pohv_system.register_heartbeat();
    let status = state.app_state.pohv_system.get_state();
    (StatusCode::OK, Json(ApiResponse::ok(status))).into_response()
}

// Topic endpoints
async fn get_topic_status(State(state): State<WebState>) -> Response {
    let status = state.app_state.topic_manager.get_status();
    (StatusCode::OK, Json(ApiResponse::ok(status))).into_response()
}

async fn set_topic(
    State(state): State<WebState>,
    Json(req): Json<TopicSetRequest>,
) -> Response {
    match state.app_state.topic_manager.set_topic(req.topic, Some(req.interval)) {
        Ok(_) => {
            let status = state.app_state.topic_manager.get_status();
            (StatusCode::OK, Json(ApiResponse::ok(status))).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::err(e)),
        )
            .into_response(),
    }
}

async fn stop_topic(State(state): State<WebState>) -> Response {
    state.app_state.topic_manager.stop();
    let status = state.app_state.topic_manager.get_status();
    (StatusCode::OK, Json(ApiResponse::ok(status))).into_response()
}

async fn get_topic_history(
    State(state): State<WebState>,
    Query(req): Query<TopicHistoryRequest>,
) -> Response {
    let limit = req.limit.unwrap_or(10) as i64;
    
    let history = if let Some(kb) = &state.app_state.knowledge_bank {
        match kb.get_recent_topics(limit).await {
            Ok(h) => h,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<Vec<(String, i64)>>::err(e))).into_response(),
        }
    } else {
        Vec::new()
    };

    (StatusCode::OK, Json(ApiResponse::ok(history))).into_response()
}

async fn get_chat_status(State(state): State<WebState>) -> Response {
    let status = state.app_state.chat_bot_status.lock().unwrap().clone();
    (StatusCode::OK, Json(ApiResponse::ok(status))).into_response()
}

// Generate Question endpoint
pub async fn generate_question(
    State(state): State<WebState>,
) -> Result<Json<String>, StatusCode> {
    let config = state.app_state.get_config();
    let model = config.ollama_model.clone();

    let prompt = config.question_generation_prompt.clone();

    match crate::ollama::ask_ollama_internal(&state.app_state, model, prompt, None).await {
        Ok(question) => Ok(Json(question.trim().to_string())),
        Err(e) => {
            eprintln!("❌ Failed to generate question: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Set User Handle endpoint
pub async fn set_user_handle(
    State(state): State<WebState>,
    Json(req): Json<SetHandleRequest>,
) -> Response {
    state.app_state.update_config(|config| {
        config.user_handle = req.handle.clone();
    });

    // Persist to disk
    let config = state.app_state.get_config();
    if let Err(e) = config.save() {
        state
            .app_state
            .log_error("set_user_handle", &format!("Failed to save config: {}", e));
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::err(e.to_string())),
        )
            .into_response();
    }

    (StatusCode::OK, Json(ApiResponse::ok(()))).into_response()
}

// Build the web server router
pub fn create_router(state: WebState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT]);

    Router::new()
        .route("/health", get(health_check))
        .route("/api/config", get(get_config))
        .route("/api/agents", get(list_agents))
        .route("/api/agents/create", post(create_agent))
        .route("/api/agents/delete", post(delete_agent))
        .route("/api/agents/reset-identity", post(reset_agent_identity))
        .route("/api/agents/stats", get(get_all_agent_stats))
        .route("/api/agents/stats/:agent_id", get(get_agent_stats))
        .route("/api/council/sessions", get(list_council_sessions))
        .route("/api/council/session", post(get_council_session))
        .route("/api/council/create", post(create_council_session))
        .route("/api/chat/send", post(send_message))
        .route("/api/chat/messages", post(get_messages))
        .route("/api/pohv/status", get(get_pohv_status))
        .route("/api/pohv/heartbeat", post(pohv_heartbeat))
        .route("/api/topic/status", get(get_topic_status))
        .route("/api/topic/set", post(set_topic))
        .route("/api/topic/stop", post(stop_topic))
        .route("/api/topic/history", get(get_topic_history))
        .route("/api/chat/status", get(get_chat_status))
        .route("/api/council/generate_question", post(generate_question))
        .route("/api/user/handle", post(set_user_handle))
        .route("/ws/chat", get(websocket_handler))
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

// ========================================
// WebSocket Handler
// ========================================

/// WebSocket upgrade handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket_connection(socket, state.app_state))
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
        .send(WsMessage::Text(welcome.to_string()))
        .await
        .is_err()
    {
        return;
    }

    // Forward broadcast messages to this client
    // We need to split the socket to handle concurrent read/write if we want to support incoming messages later
    // But for now, we just need to send messages from the broadcast channel
    
    // Note: In axum 0.7, we don't need to split explicitly for simple cases, 
    // but since we want to listen to rx loop, we should spawn a task.
    
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(chat_msg) => {
                    // Serialize and send to client
                    let json = match serde_json::to_string(&chat_msg) {
                        Ok(j) => j,
                        Err(e) => {
                            eprintln!("❌ Failed to serialize message: {}", e);
                            continue;
                        }
                    };

                    if socket.send(WsMessage::Text(json)).await.is_err() {
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
        // println!("🔌 WebSocket client disconnected");
    });
}

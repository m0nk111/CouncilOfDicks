use axum::{
    extract::{Json, Query, State},
    http::{StatusCode, Method, header},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
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
        }))),
    )
        .into_response()
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

// Council endpoints
async fn list_council_sessions(State(state): State<WebState>) -> Response {
    let sessions = state.council_manager.list_sessions().await;
    (
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({
            "sessions": sessions
        }))),
    )
        .into_response()
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
    let ollama_url = {
        let config = state
            .app_state
            .config
            .lock()
            .expect("Failed to lock config");
        config.ollama_url.clone()
    };

    match state
        .council_manager
        .create_session_with_agents(
            req.question,
            state.agent_pool.clone(),
            req.agent_ids,
            &ollama_url,
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
    let url = config.ollama_url.clone();

    let prompt = "Generate a single, short, provocative, and open-ended philosophical or ethical question for an AI council to debate. The question should be deep and require nuanced thinking. Do not include any preamble, explanation, or quotes. Just the question itself.".to_string();

    match crate::ollama::ask_ollama(&url, &model, prompt).await {
        Ok(question) => Ok(Json(question.trim().to_string())),
        Err(e) => {
            eprintln!("❌ Failed to generate question: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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

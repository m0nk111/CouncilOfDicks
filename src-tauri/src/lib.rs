pub mod agents;
mod chat;
pub mod chat_bot;
pub mod config;
pub mod council;
mod crypto;
mod deliberation;
mod http_server;
mod knowledge;
mod logger;
mod mcp;
mod metrics;
mod ollama;
mod p2p;
mod p2p_manager;
mod personalities;
mod pohv;
pub mod prompt;
mod protocol;
mod benchmarks;
mod providers;
pub mod state;
mod verdict_store;
pub mod web_server;

#[cfg(test)]
mod tests;

use benchmarks::get_benchmarks;
use config::AppConfig;
use pohv::{pohv_get_status, pohv_heartbeat};
use prompt::compose_system_prompt;
use providers::AIProvider;
use state::AppState;

// Tauri commands
#[tauri::command]
async fn ask_ollama(question: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let config = state.get_config();

    state.log_debug("ask_ollama", &format!("Question: {}", question));
    state.log_network(
        "ask_ollama",
        &format!("→ {}:{}", config.ollama_url, config.ollama_model),
    );

    // Start timing
    let start = {
        let metrics = state.metrics.lock().unwrap();
        metrics.start_request()
    };

    let tcod_prompt = format!(
        "{}\n\n# Human Question\n{}",
        compose_system_prompt(""),
        question
    );

    let result = ollama::ask_ollama(&config.ollama_url, &config.ollama_model, tcod_prompt).await;

    // Record result and sign response
    match &result {
        Ok(response) => {
            let mut metrics = state.metrics.lock().unwrap();
            metrics.record_success(start);
            state.log_success(
                "ask_ollama",
                &format!("← Response: {} chars", response.len()),
            );

            // Sign the response
            let signed = state.signing_identity.sign(response);
            state.log_debug(
                "ask_ollama",
                &format!(
                    "✍️ Response signed with key: {}...",
                    signed.public_key[..16].to_string()
                ),
            );
        }
        Err(e) => {
            let mut metrics = state.metrics.lock().unwrap();
            metrics.record_failure(start);
            state.log_error("ask_ollama", &format!("← Error: {}", e));
        }
    }

    result
}

#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>) -> AppConfig {
    state.log_debug("get_config", "Fetching config");
    state.get_config()
}

#[tauri::command]
fn set_debug(enabled: bool, state: tauri::State<'_, AppState>) {
    state.update_config(|config| {
        config.debug_enabled = enabled;
    });
    state.logger.set_debug_enabled(enabled);
    state.log_info(
        "set_debug",
        &format!("Debug mode: {}", if enabled { "ON" } else { "OFF" }),
    );
}

#[tauri::command]
fn get_metrics(state: tauri::State<'_, AppState>) -> metrics::PerformanceMetrics {
    let metrics = state.metrics.lock().unwrap();
    state.log_debug("get_metrics", "Fetching metrics");
    metrics.get_metrics()
}

#[tauri::command]
async fn p2p_start(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state.log_info("p2p_start", "Starting P2P network");
    let result = state.p2p_manager.start().await;

    match &result {
        Ok(msg) => state.log_success("p2p_start", msg),
        Err(e) => state.log_error("p2p_start", &format!("Failed: {}", e)),
    }

    result
}

#[tauri::command]
async fn p2p_stop(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state.log_info("p2p_stop", "Stopping P2P network");
    let result = state.p2p_manager.stop().await;

    match &result {
        Ok(msg) => state.log_success("p2p_stop", msg),
        Err(e) => state.log_error("p2p_stop", &format!("Failed: {}", e)),
    }

    result
}

#[tauri::command]
async fn p2p_status(
    state: tauri::State<'_, AppState>,
) -> Result<p2p_manager::NetworkStatus, String> {
    state.log_debug("p2p_status", "Fetching P2P status");
    Ok(state.p2p_manager.status().await)
}

// Council session commands
#[tauri::command]
async fn council_create_session(
    state: tauri::State<'_, AppState>,
    question: String,
) -> Result<String, String> {
    state.log_info(
        "council_create_session",
        &format!("Creating session: {}", question),
    );
    let session_id = state.council_manager.create_session(question).await;
    state.log_success(
        "council_create_session",
        &format!("Session created: {}", session_id),
    );
    Ok(session_id)
}

#[tauri::command]
async fn council_create_session_with_agents(
    state: tauri::State<'_, AppState>,
    question: String,
    agent_ids: Vec<String>,
) -> Result<String, String> {
    state.log_info(
        "council_agents",
        &format!("Creating session with {} agents", agent_ids.len()),
    );

    let config = state.get_config();
    let session_id = state
        .council_manager
        .create_session_with_agents(
            question,
            state.agent_pool.clone(),
            agent_ids,
            &config.ollama_url,
        )
        .await?;

    state.log_success(
        "council_agents",
        &format!("Session {} created, agents responded", session_id),
    );
    Ok(session_id)
}

#[tauri::command]
async fn council_get_session(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<protocol::CouncilSession, String> {
    state.log_debug(
        "council_get_session",
        &format!("Fetching session: {}", session_id),
    );
    state
        .council_manager
        .get_session(&session_id)
        .await
        .ok_or_else(|| "Session not found".to_string())
}

#[tauri::command]
async fn council_list_sessions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<protocol::CouncilSession>, String> {
    state.log_debug("council_list_sessions", "Listing all sessions");
    Ok(state.council_manager.list_sessions().await)
}

#[tauri::command]
async fn council_add_response(
    state: tauri::State<'_, AppState>,
    session_id: String,
    model_name: String,
    response: String,
    peer_id: String,
) -> Result<String, String> {
    state.log_debug(
        "council_add_response",
        &format!(
            "Adding response from {} to session {}",
            model_name, session_id
        ),
    );

    // Sign the response
    let signed = state.signing_identity.sign(&response);
    state.log_debug(
        "council_add_response",
        &format!(
            "✍️ Signed with key: {}...",
            signed.public_key[..16].to_string()
        ),
    );

    state
        .council_manager
        .add_response(
            &session_id,
            model_name,
            response,
            peer_id,
            Some(signed.signature),
            Some(signed.public_key),
        )
        .await?;

    Ok("Response added and signed".to_string())
}

#[tauri::command]
async fn council_start_voting(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<String, String> {
    state.log_info(
        "council_start_voting",
        &format!("Starting voting phase for session {}", session_id),
    );

    state
        .council_manager
        .start_commitment_phase(&session_id)
        .await?;

    Ok("Voting phase started".to_string())
}

#[tauri::command]
async fn council_calculate_consensus(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<Option<String>, String> {
    state.log_info(
        "council_calculate_consensus",
        &format!("Calculating consensus for session {}", session_id),
    );

    let consensus = state
        .council_manager
        .calculate_consensus(&session_id)
        .await?;

    if let Some(ref result) = consensus {
        state.log_success(
            "council_calculate_consensus",
            &format!("Consensus reached: {}", result),
        );

        if let Some(store) = &state.verdict_store {
            if let Some(session) = state.council_manager.get_session(&session_id).await {
                if let Err(e) = store.store_verdict(&session).await {
                    state.log_error(
                        "verdict_store",
                        &format!("Failed to persist verdict {}: {}", session_id, e),
                    );
                } else {
                    state.log_success(
                        "verdict_store",
                        &format!("Stored verdict for session {}", session_id),
                    );
                }
            }
        }
    } else {
        state.log_info("council_calculate_consensus", "No consensus reached");
    }

    Ok(consensus)
}

// Crypto commands
#[tauri::command]
async fn get_public_key(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state.log_debug("get_public_key", "Fetching public key");
    Ok(state.signing_identity.public_key_base64())
}

#[tauri::command]
async fn verify_signature(
    content: String,
    signature: String,
    public_key: String,
    timestamp: u64,
) -> Result<bool, String> {
    use crate::crypto::{verify_signed_message, SignedMessage};

    let signed_msg = SignedMessage {
        content,
        signature,
        public_key,
        timestamp,
    };

    verify_signed_message(&signed_msg)
}

#[tauri::command]
async fn get_key_fingerprint(state: tauri::State<'_, AppState>) -> Result<String, String> {
    use crate::crypto::public_key_fingerprint;
    let pubkey = state.signing_identity.public_key_base64();
    public_key_fingerprint(&pubkey)
}

// MCP server commands
#[tauri::command]
async fn mcp_start(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state.log_info("mcp_start", "Starting MCP server");
    let result = state.mcp_server.start().await;

    match &result {
        Ok(msg) => state.log_success("mcp_start", msg),
        Err(e) => state.log_error("mcp_start", &format!("Failed: {}", e)),
    }

    result
}

#[tauri::command]
async fn mcp_stop(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state.log_info("mcp_stop", "Stopping MCP server");
    let result = state.mcp_server.stop().await;

    match &result {
        Ok(msg) => state.log_success("mcp_stop", msg),
        Err(e) => state.log_error("mcp_stop", &format!("Failed: {}", e)),
    }

    result
}

#[tauri::command]
async fn mcp_status(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    state.log_debug("mcp_status", "Checking MCP server status");
    Ok(state.mcp_server.is_running().await)
}

// Deliberation commands
#[tauri::command]
async fn start_deliberation(
    question: String,
    member_count: usize,
    max_rounds: usize,
    state: tauri::State<'_, AppState>,
) -> Result<deliberation::DeliberationResult, String> {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    state.log_info(
        "start_deliberation",
        &format!(
            "Starting deliberation with {} members, max {} rounds",
            member_count, max_rounds
        ),
    );

    // Create deliberation engine
    let config = state.get_config();
    let ollama_client = Arc::new(Mutex::new(ollama::OllamaClient::new(
        config.clone(),
        state.logger.clone(),
    )));

    let engine = deliberation::DeliberationEngine::new(state.logger.clone(), ollama_client);

    // Create council members with personalities
    let members = personalities::create_council_members(&config.ollama_model, member_count);

    state.log_debug(
        "start_deliberation",
        &format!(
            "Council members: {:?}",
            members.iter().map(|m| &m.name).collect::<Vec<_>>()
        ),
    );

    // Start deliberation
    let result = engine
        .start_deliberation(question, members, max_rounds)
        .await;

    match &result {
        Ok(res) => state.log_success(
            "start_deliberation",
            &format!(
                "Completed {} rounds, consensus: {}",
                res.rounds.len(),
                res.consensus.is_some()
            ),
        ),
        Err(e) => state.log_error("start_deliberation", &format!("Failed: {}", e)),
    }

    result
}

#[tauri::command]
async fn get_personalities() -> Result<Vec<personalities::Personality>, String> {
    Ok(personalities::get_default_personalities())
}

#[tauri::command]
async fn create_custom_council(
    model_name: String,
    personality_names: Vec<String>,
) -> Result<Vec<deliberation::CouncilMember>, String> {
    let personalities = personalities::get_default_personalities();
    let mut members = Vec::new();

    for name in personality_names {
        if let Some(personality) = personalities.iter().find(|p| p.name == name) {
            members.push(deliberation::CouncilMember {
                name: personality.name.clone(),
                model: model_name.clone(),
                personality: personality.name.clone(),
                system_prompt: personality.system_prompt.clone(),
            });
        }
    }

    if members.is_empty() {
        return Err("No valid personalities found".to_string());
    }

    Ok(members)
}

// Knowledge Bank commands
#[tauri::command]
async fn kb_store_deliberation(
    result: deliberation::DeliberationResult,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    state.log_info(
        "kb_store",
        &format!("Storing deliberation: {}", result.session_id),
    );

    if let Some(kb) = &state.knowledge_bank {
        kb.store_deliberation(&result).await?;
        state.log_success("kb_store", "Deliberation stored with embeddings");
        Ok("Stored successfully".to_string())
    } else {
        Err("Knowledge bank not initialized".to_string())
    }
}

#[tauri::command]
async fn kb_search(
    query: String,
    limit: usize,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<knowledge::SearchResult>, String> {
    state.log_info("kb_search", &format!("Searching: {}", query));

    if let Some(kb) = &state.knowledge_bank {
        let results = kb.semantic_search(&query, limit).await?;
        state.log_success("kb_search", &format!("Found {} results", results.len()));
        Ok(results)
    } else {
        Err("Knowledge bank not initialized".to_string())
    }
}

#[tauri::command]
async fn kb_get_rag_context(
    question: String,
    top_k: usize,
    state: tauri::State<'_, AppState>,
) -> Result<knowledge::RAGContext, String> {
    state.log_info("kb_rag", &format!("Building RAG context for: {}", question));

    if let Some(kb) = &state.knowledge_bank {
        let context = kb.build_rag_context(&question, top_k).await?;
        state.log_success(
            "kb_rag",
            &format!(
                "Built context with {} decisions",
                context.relevant_decisions.len()
            ),
        );
        Ok(context)
    } else {
        Err("Knowledge bank not initialized".to_string())
    }
}

#[tauri::command]
async fn kb_list_all(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<(String, String, bool)>, String> {
    if let Some(kb) = &state.knowledge_bank {
        kb.list_all().await
    } else {
        Err("Knowledge bank not initialized".to_string())
    }
}

#[tauri::command]
async fn verdict_list_recent(
    state: tauri::State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<verdict_store::CouncilVerdictRecord>, String> {
    let max = limit.unwrap_or(20);
    if let Some(store) = &state.verdict_store {
        store.list_recent(max).await
    } else {
        Err("Verdict storage not initialized".to_string())
    }
}

#[tauri::command]
async fn verdict_get(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<Option<verdict_store::CouncilVerdictRecord>, String> {
    if let Some(store) = &state.verdict_store {
        store.get(&session_id).await
    } else {
        Err("Verdict storage not initialized".to_string())
    }
}

// Chat commands
#[tauri::command]
fn chat_send_message(
    channel: String,
    author: String,
    author_type: String,
    content: String,
    signature: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    use chat::{AuthorType, ChannelType, Message};

    let channel_type =
        ChannelType::from_str(&channel).ok_or_else(|| format!("Invalid channel: {}", channel))?;

    let author_type = match author_type.as_str() {
        "human" => AuthorType::Human,
        "ai" => AuthorType::AI,
        "system" => AuthorType::System,
        _ => return Err(format!("Invalid author type: {}", author_type)),
    };

    let mut message = Message::new(channel_type, author, author_type, content);

    if let Some(sig) = signature {
        message = message.with_signature(sig);
    }

    state.log_debug(
        "chat",
        &format!("Sending message to #{}: {}", channel, message.id),
    );

    let result = state.channel_manager.send_message(message.clone());

    // Broadcast to WebSocket clients
    if result.is_ok() {
        let _ = state.websocket_broadcast.send(message);
    }

    result
}

#[tauri::command]
fn chat_get_messages(
    channel: String,
    limit: usize,
    offset: usize,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<chat::Message>, String> {
    use chat::ChannelType;

    let channel_type =
        ChannelType::from_str(&channel).ok_or_else(|| format!("Invalid channel: {}", channel))?;

    state
        .channel_manager
        .get_messages(channel_type, limit, offset)
}

#[tauri::command]
fn chat_add_reaction(
    channel: String,
    message_id: String,
    emoji: String,
    author: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    use chat::ChannelType;

    let channel_type =
        ChannelType::from_str(&channel).ok_or_else(|| format!("Invalid channel: {}", channel))?;

    state
        .channel_manager
        .add_reaction(channel_type, &message_id, emoji, author)
}

#[tauri::command]
fn chat_get_message_count(
    channel: String,
    state: tauri::State<'_, AppState>,
) -> Result<usize, String> {
    use chat::ChannelType;

    let channel_type =
        ChannelType::from_str(&channel).ok_or_else(|| format!("Invalid channel: {}", channel))?;

    state.channel_manager.message_count(channel_type)
}

#[tauri::command]
async fn chat_check_duplicate(
    question: String,
    state: tauri::State<'_, AppState>,
) -> Result<chat::DuplicateCheckResult, String> {
    state.log_debug("chat", &format!("🔍 Checking for duplicate: {}", question));

    if let Some(ref duplicate_filter) = state.duplicate_filter {
        let result = duplicate_filter
            .check_duplicate(&question)
            .await
            .map_err(|e| format!("Failed to check duplicate: {}", e))?;

        if result.is_duplicate {
            state.log_info(
                "chat",
                &format!(
                    "⛔ Duplicate detected: score={:.2}, session={}",
                    result.similarity_score,
                    result
                        .existing_session_id
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                ),
            );
        } else if result.similarity_score > 0.70 {
            state.log_debug(
                "chat",
                &format!(
                    "💡 Related question found: score={:.2}",
                    result.similarity_score
                ),
            );
        }

        Ok(result)
    } else {
        Err("Duplicate filter not available (Knowledge Bank not initialized)".to_string())
    }
}

#[tauri::command]
async fn chat_check_rate_limit(
    user_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<chat::RateLimitResult, String> {
    state.log_debug(
        "chat",
        &format!("⏱️ Checking rate limit for user: {}", user_id),
    );

    let result = state.rate_limiter.check_rate_limit(&user_id);

    if !result.allowed {
        state.log_info(
            "chat",
            &format!(
                "🚫 Rate limit exceeded for {}: {}",
                user_id,
                result.reason.as_ref().unwrap_or(&"Unknown".to_string())
            ),
        );
    }

    Ok(result)
}

#[tauri::command]
async fn chat_record_question(
    user_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.rate_limiter.record_question(&user_id);
    state.log_debug(
        "chat",
        &format!("✅ Recorded question for user: {}", user_id),
    );
    Ok(())
}

#[tauri::command]
async fn chat_check_spam(
    user_id: String,
    message: String,
    state: tauri::State<'_, AppState>,
) -> Result<chat::SpamCheckResult, String> {
    state.log_debug("chat", &format!("🛡️ Checking spam for user: {}", user_id));

    let result = state.spam_detector.check_spam(&user_id, &message);

    if result.is_spam {
        state.log_info(
            "chat",
            &format!(
                "⚠️ Spam detected: score={:.2}, level={:?}, reasons={:?}",
                result.spam_score, result.spam_level, result.reasons
            ),
        );
    }

    Ok(result)
}

#[tauri::command]
async fn chat_record_message(
    user_id: String,
    message: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.spam_detector.record_message(&user_id, &message);
    state.log_debug(
        "chat",
        &format!("✅ Recorded message for user: {}", user_id),
    );
    Ok(())
}

// Agent pool management commands
#[tauri::command]
async fn agent_add(
    state: tauri::State<'_, AppState>,
    name: String,
    model: String,
    system_prompt: String,
) -> Result<String, String> {
    let agent = agents::Agent::new(name, model, system_prompt);
    let agent_id = state.agent_pool.add_agent(agent).await?;
    state.log_success("agent", &format!("Added agent: {}", agent_id));
    Ok(agent_id)
}

#[tauri::command]
async fn agent_remove(state: tauri::State<'_, AppState>, agent_id: String) -> Result<(), String> {
    state.agent_pool.remove_agent(&agent_id).await?;
    state.log_success("agent", &format!("Removed agent: {}", agent_id));
    Ok(())
}

#[tauri::command]
async fn agent_update(
    state: tauri::State<'_, AppState>,
    agent: agents::Agent,
) -> Result<(), String> {
    state.agent_pool.update_agent(agent.clone()).await?;
    state.log_success("agent", &format!("Updated agent: {}", agent.id));
    Ok(())
}

#[tauri::command]
async fn agent_list(state: tauri::State<'_, AppState>) -> Result<Vec<agents::Agent>, String> {
    Ok(state.agent_pool.list_agents().await)
}

#[tauri::command]
async fn agent_get(
    state: tauri::State<'_, AppState>,
    agent_id: String,
) -> Result<agents::Agent, String> {
    state.agent_pool.get_agent(&agent_id).await
}

#[tauri::command]
async fn agent_list_active(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<agents::Agent>, String> {
    Ok(state.agent_pool.list_active_agents().await)
}

#[tauri::command]
async fn agent_get_tools() -> Result<Vec<agents::Tool>, String> {
    Ok(agents::Tool::standard_tools())
}

// Provider management commands
#[tauri::command]
async fn provider_add(
    config: providers::config::ProviderConfig,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    state.log_info(
        "provider_add",
        &format!("Adding provider: {}", config.username),
    );

    // Validate config
    providers::config::validate_provider_config(&config)?;

    // Load current config
    let config_path = "providers.json";
    let mut providers_config =
        providers::config::ProvidersConfig::load(config_path).unwrap_or_default();

    // Add/update provider
    providers_config.upsert_provider(config.clone());

    // Save config
    providers_config.save(config_path)?;

    state.log_success(
        "provider_add",
        &format!("Added provider: {}", config.username),
    );
    Ok(config.id)
}

#[tauri::command]
fn provider_list(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<providers::config::ProviderConfig>, String> {
    state.log_info("provider_list", "Listing providers");

    let config_path = "providers.json";
    let providers_config =
        providers::config::ProvidersConfig::load(config_path).unwrap_or_default();

    Ok(providers_config.providers)
}

#[tauri::command]
fn provider_remove(id: String, state: tauri::State<'_, AppState>) -> Result<bool, String> {
    state.log_info("provider_remove", &format!("Removing provider: {}", id));

    let config_path = "providers.json";
    let mut providers_config =
        providers::config::ProvidersConfig::load(config_path).unwrap_or_default();

    let removed = providers_config.remove_provider(&id);

    if removed {
        providers_config.save(config_path)?;
        state.log_success("provider_remove", &format!("Removed provider: {}", id));
    }

    Ok(removed)
}

#[tauri::command]
async fn provider_test_connection(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<providers::ProviderHealth, String> {
    state.log_info("provider_test", &format!("Testing provider: {}", id));

    // Load config
    let config_path = "providers.json";
    let providers_config =
        providers::config::ProvidersConfig::load(config_path).unwrap_or_default();

    let provider_config = providers_config
        .get_provider(&id)
        .ok_or(format!("Provider '{}' not found", id))?;

    // Create provider instance and test
    match &provider_config.config {
        providers::config::ProviderSpecificConfig::Ollama {
            base_url,
            default_model,
            ..
        } => {
            let provider = providers::OllamaProvider::new(
                base_url.clone(),
                default_model.clone(),
                state.logger.clone(),
            );

            let health = provider
                .health_check()
                .await
                .map_err(|e| format!("Health check failed: {}", e))?;
            state.log_success(
                "provider_test",
                &format!("Provider {} health: {:?}", id, health.healthy),
            );
            Ok(health)
        }
        _ => Err("Provider type not yet supported for testing".to_string()),
    }
}

#[tauri::command]
fn provider_set_default(
    provider_id: String,
    purpose: String, // "generation" or "embedding"
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.log_info(
        "provider_set_default",
        &format!("Setting {} default to: {}", purpose, provider_id),
    );

    let config_path = "providers.json";
    let mut providers_config =
        providers::config::ProvidersConfig::load(config_path).unwrap_or_default();

    match purpose.as_str() {
        "generation" => {
            providers_config.default_generation_provider = Some(provider_id);
        }
        "embedding" => {
            providers_config.default_embedding_provider = Some(provider_id);
        }
        _ => return Err(format!("Invalid purpose: {}", purpose)),
    }

    providers_config.save(config_path)?;
    state.log_success("provider_set_default", "Default provider updated");

    Ok(())
}

#[tauri::command]
async fn provider_generate_username(
    model_name: String,
    provider_name: String,
) -> Result<String, String> {
    providers::config::generate_username_from_model(&model_name, &provider_name).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize app state
    let state = AppState::new();
    let config = state.get_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            ask_ollama,
            get_config,
            set_debug,
            get_metrics,
            p2p_start,
            p2p_stop,
            p2p_status,
            council_create_session,
            council_create_session_with_agents,
            council_get_session,
            council_list_sessions,
            council_add_response,
            council_start_voting,
            council_calculate_consensus,
            get_public_key,
            verify_signature,
            get_key_fingerprint,
            mcp_start,
            mcp_stop,
            mcp_status,
            start_deliberation,
            get_personalities,
            create_custom_council,
            kb_store_deliberation,
            kb_search,
            kb_get_rag_context,
            kb_list_all,
            verdict_list_recent,
            verdict_get,
            pohv_heartbeat,
            pohv_get_status,
            provider_add,
            provider_list,
            provider_remove,
            provider_test_connection,
            provider_set_default,
            provider_generate_username,
            chat_send_message,
            chat_get_messages,
            chat_add_reaction,
            chat_get_message_count,
            chat_check_duplicate,
            chat_check_rate_limit,
            chat_record_question,
            chat_check_spam,
            chat_record_message,
            agent_add,
            agent_remove,
            agent_update,
            agent_list,
            agent_get,
            agent_list_active,
            agent_get_tools,
            get_benchmarks
        ])
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            println!("╔════════════════════════════════════════╗");
            println!("║   Council Of Dicks - MVP Foundation    ║");
            println!("╚════════════════════════════════════════╝");
            println!("✅ App initialized");
            println!("🔥 NR5 IS ALIVE at {}", config.ollama_url);
            println!("🤖 Model: {}", config.ollama_model);
            println!(
                "🐛 Debug mode: {}",
                if config.debug_enabled { "ON" } else { "OFF" }
            );
            println!("\n🚀 And awaaaay we go!\n");

            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|err| {
            eprintln!("\n❌ Failed to start Tauri application");
            eprintln!("Error: {}", err);

            if err.to_string().contains("gtk") || err.to_string().contains("GTK") {
                eprintln!("\n💡 This appears to be a GTK/Display issue.");
                eprintln!("   Possible causes:");
                eprintln!("   • Running on headless server without X11/Wayland");
                eprintln!("   • Missing DISPLAY environment variable");
                eprintln!("   • GTK libraries not properly installed");
                eprintln!("\n   Solutions:");
                eprintln!("   • Run on a system with a desktop environment");
                eprintln!("   • Use Xvfb for headless testing: xvfb-run pnpm tauri dev");
                eprintln!("   • Test backend only: ./scripts/test-backend.sh");
            }

            std::process::exit(1);
        });
}

/// Run in HTTP server mode (no GUI)
/// Usage: ./app --server or ./app serve
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments
    let args: Vec<String> = std::env::args().collect();
    let mut port = 8080u16;
    let mut host = "127.0.0.1".to_string();

    // Simple argument parsing
    for i in 1..args.len() {
        if args[i] == "--port" || args[i] == "-p" {
            if let Some(p) = args.get(i + 1) {
                port = p.parse().unwrap_or(8080);
            }
        } else if args[i] == "--host" || args[i] == "-h" {
            if let Some(h) = args.get(i + 1) {
                host = h.clone();
            }
        }
    }

    println!("╔════════════════════════════════════════╗");
    println!("║   Council Of Dicks - HTTP Server Mode  ║");
    println!("╚════════════════════════════════════════╝");

    // Initialize app state
    let state = AppState::new();
    let config = state.get_config();

    println!("✅ App state initialized");
    println!("🔥 NR5 IS ALIVE at {}", config.ollama_url);
    println!("🤖 Model: {}", config.ollama_model);
    println!(
        "🐛 Debug mode: {}",
        if config.debug_enabled { "ON" } else { "OFF" }
    );

    // Create HTTP server
    let http_config = http_server::HttpServerConfig {
        port,
        host: host.clone(),
        enable_cors: true,
    };

    let server = http_server::HttpServer::new(http_config, std::sync::Arc::new(state));

    println!("\n🌐 Starting HTTP API server...");
    println!("   • Web UI: http://{}:{}", host, port);
    println!("   • API: http://{}:{}/api/*", host, port);
    println!("   • Health: http://{}:{}/health", host, port);
    println!("   • WebSocket: ws://{}:{}/ws/chat", host, port);
    println!("\n🚀 And awaaaay we go!\n");

    // Start server (blocks until shutdown)
    server.start().await?;

    Ok(())
}

mod config;
mod council;
mod crypto;
mod deliberation;
mod mcp;
mod ollama;
mod personalities;
mod state;
mod logger;
mod metrics;
mod p2p;
mod protocol;
mod p2p_manager;

#[cfg(test)]
mod tests;

use config::AppConfig;
use state::AppState;

// Tauri commands
#[tauri::command]
async fn ask_ollama(
    question: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let config = state.get_config();
    
    state.log_debug("ask_ollama", &format!("Question: {}", question));
    state.log_network("ask_ollama", &format!("→ {}:{}", config.ollama_url, config.ollama_model));

    // Start timing
    let start = {
        let metrics = state.metrics.lock().unwrap();
        metrics.start_request()
    };

    let result = ollama::ask_ollama(&config.ollama_url, &config.ollama_model, question).await;

    // Record result and sign response
    match &result {
        Ok(response) => {
            let mut metrics = state.metrics.lock().unwrap();
            metrics.record_success(start);
            state.log_success("ask_ollama", &format!("← Response: {} chars", response.len()));
            
            // Sign the response
            let signed = state.signing_identity.sign(response);
            state.log_debug("ask_ollama", &format!("✍️ Response signed with key: {}...", 
                signed.public_key[..16].to_string()));
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
    state.log_info("set_debug", &format!("Debug mode: {}", if enabled { "ON" } else { "OFF" }));
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
async fn p2p_status(state: tauri::State<'_, AppState>) -> Result<p2p_manager::NetworkStatus, String> {
    state.log_debug("p2p_status", "Fetching P2P status");
    Ok(state.p2p_manager.status().await)
}

// Council session commands
#[tauri::command]
async fn council_create_session(state: tauri::State<'_, AppState>, question: String) -> Result<String, String> {
    state.log_info("council_create_session", &format!("Creating session: {}", question));
    let session_id = state.council_manager.create_session(question).await;
    state.log_success("council_create_session", &format!("Session created: {}", session_id));
    Ok(session_id)
}

#[tauri::command]
async fn council_get_session(state: tauri::State<'_, AppState>, session_id: String) -> Result<protocol::CouncilSession, String> {
    state.log_debug("council_get_session", &format!("Fetching session: {}", session_id));
    state.council_manager
        .get_session(&session_id)
        .await
        .ok_or_else(|| "Session not found".to_string())
}

#[tauri::command]
async fn council_list_sessions(state: tauri::State<'_, AppState>) -> Result<Vec<protocol::CouncilSession>, String> {
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
    state.log_debug("council_add_response", &format!("Adding response from {} to session {}", model_name, session_id));
    
    // Sign the response
    let signed = state.signing_identity.sign(&response);
    state.log_debug("council_add_response", &format!("✍️ Signed with key: {}...", 
        signed.public_key[..16].to_string()));
    
    state.council_manager
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
async fn council_start_voting(state: tauri::State<'_, AppState>, session_id: String) -> Result<String, String> {
    state.log_info("council_start_voting", &format!("Starting voting phase for session {}", session_id));
    
    state.council_manager
        .start_commitment_phase(&session_id)
        .await?;
    
    Ok("Voting phase started".to_string())
}

#[tauri::command]
async fn council_calculate_consensus(state: tauri::State<'_, AppState>, session_id: String) -> Result<Option<String>, String> {
    state.log_info("council_calculate_consensus", &format!("Calculating consensus for session {}", session_id));
    
    let consensus = state.council_manager
        .calculate_consensus(&session_id)
        .await?;
    
    if let Some(ref result) = consensus {
        state.log_success("council_calculate_consensus", &format!("Consensus reached: {}", result));
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
    
    state.log_info("start_deliberation", &format!("Starting deliberation with {} members, max {} rounds", member_count, max_rounds));
    
    // Create deliberation engine
    let config = state.get_config();
    let ollama_client = Arc::new(Mutex::new(ollama::OllamaClient::new(
        config.clone(),
        state.logger.clone(),
    )));
    
    let engine = deliberation::DeliberationEngine::new(
        state.logger.clone(),
        ollama_client,
    );
    
    // Create council members with personalities
    let members = personalities::create_council_members(&config.ollama_model, member_count);
    
    state.log_debug("start_deliberation", &format!("Council members: {:?}", 
        members.iter().map(|m| &m.name).collect::<Vec<_>>()));
    
    // Start deliberation
    let result = engine.start_deliberation(question, members, max_rounds).await;
    
    match &result {
        Ok(res) => state.log_success("start_deliberation", 
            &format!("Completed {} rounds, consensus: {}", res.rounds.len(), res.consensus.is_some())),
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
        create_custom_council
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
      println!("🐛 Debug mode: {}", if config.debug_enabled { "ON" } else { "OFF" });
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

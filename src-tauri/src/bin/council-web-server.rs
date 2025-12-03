// Standalone web server binary (no GUI required)
// Run with: cargo run --bin council-web-server

use app_lib::{
    agents::{Agent, AgentPool},
    chat_bot::ChatBot,
    council::CouncilSessionManager,
    state::AppState,
    web_server::start_web_server,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏛️ Council Of Dicks - Web Server Mode");
    println!("======================================\n");

    // Initialize app state
    let app_state = Arc::new(AppState::new());
    let config = app_state.get_config();

    println!("✅ Config loaded:");
    println!("   Ollama URL: {}", config.ollama_url);
    println!("   Model: {}", config.ollama_model);
    println!("   Debug: {}\n", config.debug_enabled);

    // Initialize components
    let council_manager = Arc::new(CouncilSessionManager::new(None));
    let agent_pool = Arc::new(AgentPool::new());

    println!("✅ Council manager initialized");
    println!("✅ Agent pool initialized");

    ensure_default_agent(&agent_pool, &app_state, &config.ollama_model).await;

    let app_state_clone = Arc::clone(&app_state);
    let agent_pool_clone = Arc::clone(&agent_pool);
    tokio::spawn(async move {
        let mut chat_bot = ChatBot::new(app_state_clone, agent_pool_clone);
        chat_bot.start_monitoring().await;
    });

    println!("✅ Chat bot enabled – listening to #general\n");

    // Start web server
    let port = 8080;
    println!("🚀 Starting web server on port {}...\n", port);

    start_web_server(app_state, council_manager, agent_pool, port).await?;

    Ok(())
}

async fn ensure_default_agent(
    agent_pool: &Arc<AgentPool>,
    app_state: &Arc<AppState>,
    default_model: &str,
) {
    if !agent_pool.list_agents().await.is_empty() {
        return;
    }

    let prompt = "You are Pragmatic Sentinel, a pragmatic guardian of the Council. Keep answers concise, cite trade-offs, and always ask for clarification when humans are vague.";

    let mut agent = Agent::new(
        "Pragmatic Sentinel".to_string(),
        default_model.to_string(),
        prompt.to_string(),
    );
    agent
        .metadata
        .insert("role".to_string(), "default_guardian".to_string());

    match agent_pool.add_agent(agent).await {
        Ok(agent_id) => {
            app_state.log_success(
                "agent",
                &format!("Seeded default Pragmatic Sentinel agent ({})", agent_id),
            );
        }
        Err(err) => {
            app_state.log_error("agent", &format!("Failed to seed default agent: {}", err));
        }
    }
}

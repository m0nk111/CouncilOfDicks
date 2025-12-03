// Standalone web server binary (no GUI required)
// Run with: cargo run --bin council-web-server

use app_lib::{
    agents::AgentPool,
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
    let app_state = Arc::new(AppState::initialize().await);
    let config = app_state.get_config();

    println!("✅ Config loaded:");
    println!("   Ollama URL: {}", config.ollama_url);
    println!("   Model: {}", config.ollama_model);
    println!("   Debug: {}\n", config.debug_enabled);

    // Initialize components from AppState
    let council_manager = Arc::clone(&app_state.council_manager);
    let agent_pool = Arc::clone(&app_state.agent_pool);

    println!("✅ Council manager initialized");
    println!("✅ Agent pool initialized");



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




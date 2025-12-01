// Standalone web server binary (no GUI required)
// Run with: cargo run --bin council-web-server

use app_lib::{
    agents::AgentPool,
    config::AppConfig,
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
    let config = AppConfig::default();
    println!("✅ Config loaded:");
    println!("   Ollama URL: {}", config.ollama_url);
    println!("   Model: {}", config.ollama_model);
    println!("   Debug: {}\n", config.debug_enabled);

    let app_state = Arc::new(AppState::new(config));
    
    // Initialize components
    let council_manager = Arc::new(CouncilSessionManager::new());
    let agent_pool = Arc::new(AgentPool::new());

    println!("✅ Council manager initialized");
    println!("✅ Agent pool initialized\n");

    // Start web server
    let port = 8080;
    println!("🚀 Starting web server on port {}...\n", port);
    
    start_web_server(app_state, council_manager, agent_pool, port).await?;

    Ok(())
}

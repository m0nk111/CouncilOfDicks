mod config;
mod ollama;
mod state;

use config::AppConfig;
use state::AppState;

// Tauri commands
#[tauri::command]
async fn ask_ollama(
    question: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let config = state.get_config();
    
    if config.debug_enabled {
        println!("🐛 [DEBUG] Received question: {}", question);
    }

    ollama::ask_ollama(&config.ollama_url, &config.ollama_model, question).await
}

#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>) -> AppConfig {
    state.get_config()
}

#[tauri::command]
fn set_debug(enabled: bool, state: tauri::State<'_, AppState>) {
    state.update_config(|config| {
        config.debug_enabled = enabled;
        println!("🔧 [CONFIG] Debug mode: {}", if enabled { "ON" } else { "OFF" });
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // Initialize app state
  let config = AppConfig::default();
  let state = AppState::new(config.clone());

  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .manage(state)
    .invoke_handler(tauri::generate_handler![
        ask_ollama,
        get_config,
        set_debug
    ])
    .setup(|app| {
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
    .expect("error while running tauri application");
}

mod config;
mod ollama;
mod state;
mod logger;
mod metrics;
mod p2p;
mod protocol;

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

    // Record result
    match &result {
        Ok(response) => {
            let mut metrics = state.metrics.lock().unwrap();
            metrics.record_success(start);
            state.log_success("ask_ollama", &format!("← Response: {} chars", response.len()));
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
        set_debug,
        get_metrics
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

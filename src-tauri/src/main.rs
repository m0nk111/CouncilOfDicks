// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() {
  // Check for CLI arguments
  let args: Vec<String> = std::env::args().collect();
  
  // Run in HTTP server mode if --server or serve flag is present
  if args.len() > 1 && (args[1] == "--server" || args[1] == "serve" || args[1] == "--serve") {
    if let Err(e) = app_lib::run_server().await {
      eprintln!("❌ HTTP server error: {}", e);
      std::process::exit(1);
    }
  } else {
    // Run in native Tauri GUI mode (default)
    app_lib::run();
  }
}

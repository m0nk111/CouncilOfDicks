use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaResponse {
    response: String,
}

#[tauri::command]
async fn ask_ollama(question: String) -> Result<String, String> {
    println!("🐛 [DEBUG] Received question: {}", question);
    
    let ollama_url = "http://192.168.1.5:11434/api/generate";
    let request_body = OllamaRequest {
        model: "qwen2.5-coder:7b".to_string(),
        prompt: question.clone(),
        stream: false,
    };

    println!("🐛 [DEBUG] Sending request to NR5 (192.168.1.5:11434)...");

    let client = reqwest::Client::new();
    let response = client
        .post(ollama_url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to connect to Ollama: {}", e);
            println!("❌ [ERROR] {}", error_msg);
            error_msg
        })?;

    println!("✅ [DEBUG] Got response from NR5!");

    let ollama_response: OllamaResponse = response
        .json()
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to parse Ollama response: {}", e);
            println!("❌ [ERROR] {}", error_msg);
            error_msg
        })?;

    println!("🎉 [DEBUG] Response parsed successfully!");
    Ok(ollama_response.response)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .invoke_handler(tauri::generate_handler![ask_ollama])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      println!("✅ Council Of Dicks initialized!");
      println!("🔥 NR5 IS ALIVE at 192.168.1.5:11434");
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

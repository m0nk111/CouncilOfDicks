# Tauri Backend - Rust

## Structure

```
src-tauri/
├── src/
│   ├── main.rs          # Entry point
│   ├── lib.rs           # Main app logic + Tauri commands
│   ├── config.rs        # Configuration management
│   ├── state.rs         # App state management
│   └── ollama.rs        # Ollama API client
├── Cargo.toml           # Rust dependencies
└── tauri.conf.json      # Tauri configuration
```

## Modules

### `config.rs`
- `AppConfig`: Global configuration structure
- Default settings for Ollama URL, model, debug mode

### `state.rs`
- `AppState`: Thread-safe state management using Arc<Mutex>
- Config updates at runtime

### `ollama.rs`
- `ask_ollama()`: HTTP client for Ollama API
- Error handling with descriptive messages
- Timeout support (120s)

### `lib.rs`
- Tauri commands:
  - `ask_ollama`: Send question to Ollama
  - `get_config`: Get current configuration
  - `set_debug`: Toggle debug mode
- App initialization with state management

## Debug Logging

Debug logs use emoji prefixes:
- 🐛 General debug
- 🔍 Inspection
- 📡 Network operations
- ✅ Success
- ❌ Errors
- 🔧 Configuration changes

## Development

```bash
# Run in dev mode (hot reload)
pnpm tauri dev

# Build for production
pnpm tauri build

# Run tests
cargo test --manifest-path=src-tauri/Cargo.toml

# Check code
cargo clippy --manifest-path=src-tauri/Cargo.toml

# Format code
cargo fmt --manifest-path=src-tauri/Cargo.toml
```

## Dependencies

- **tauri**: Cross-platform framework
- **reqwest**: HTTP client for Ollama
- **serde**: Serialization/deserialization
- **tokio**: Async runtime

# Headless Development

## Problem
Tauri requires a GUI backend (GTK on Linux, Windows APIs on Windows, Cocoa on macOS). This fails on headless servers:

```
Failed to initialize GTK backend!
```

## Solutions

### Option 1: Test Backend Only (Recommended for CI/Headless)

```bash
./scripts/test-backend.sh
```

This runs Rust unit tests without GUI.

### Option 2: Use Xvfb (Virtual Display)

Install Xvfb:
```bash
sudo apt install xvfb
```

Run with virtual display:
```bash
xvfb-run -a pnpm tauri dev
```

### Option 3: Develop on Desktop System

Transfer code to system with display:
```bash
# On headless server
git push

# On desktop
git pull
pnpm install
pnpm tauri dev
```

## Backend Testing Without GUI

All Tauri commands can be tested without GUI by creating unit tests in `src-tauri/src/`.

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_integration() {
        let config = AppConfig::default();
        let result = ollama::ask_ollama(
            &config.ollama_url,
            &config.ollama_model,
            "Test question".to_string()
        ).await;
        
        assert!(result.is_ok());
    }
}
```

Run tests:
```bash
cd src-tauri
cargo test
```

## Current Status

✅ Backend compiled successfully  
✅ Logger, Metrics, State modules working  
✅ Ollama integration implemented  
❌ GUI requires display (expected on headless server)

**Next Step:** Test full UI on desktop system or use Xvfb for CI.

## 📚 Related Documentation

- **[DOCKER.md](DOCKER.md)**: How to run the application in a containerized environment.
- **[AI_PROVIDERS.md](AI_PROVIDERS.md)**: Configuring providers in a headless setup.

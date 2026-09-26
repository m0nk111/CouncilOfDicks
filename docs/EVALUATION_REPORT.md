# Council Of Dicks - Evaluation Report

**Date**: December 1, 2025
**Version Evaluated**: v0.6.0-alpha
**Auditor**: GitHub Copilot (Senior Architect & Security Auditor)

## 1. Executive Summary

The "Council Of Dicks" project demonstrates a solid foundation for a decentralized AI consensus network. The core P2P architecture using `libp2p` and the cryptographic implementation using `ed25519-dalek` are technically sound. The integration of Ollama and the new Agent Pool system shows good progress towards the project's vision.

However, the codebase currently suffers from **critical stability and security risks** that make it unsuitable for production use. The widespread use of `unwrap()` in the backend guarantees crashes under error conditions. The lack of a Content Security Policy (CSP) in the frontend configuration is a severe security oversight.

**Overall Health Score**: C- (Promising Architecture, Dangerous Implementation)

---

## 2. Critical Issues (Must Fix)

### 2.1. Unsafe Error Handling (`unwrap()` usage)
The backend code is riddled with `unwrap()` calls on `Result` and `Option` types. This means the application **will panic and crash** if:
*   A session ID is not found.
*   A P2P network operation fails.
*   A lock is poisoned.

**Locations**:
*   `src-tauri/src/council.rs`: `manager.get_session(&session_id).await.unwrap()` (Lines 289, 307, 343). **Risk**: Crashing when querying a non-existent session.
*   `src-tauri/src/p2p_manager.rs`: `manager.start().await.unwrap()` (Line 172). **Risk**: Crash if port is already in use.
*   `src-tauri/src/agents.rs`: `pool.add_agent(...).await.unwrap()` (Line 276). **Risk**: Crash if agent addition fails.
*   `src-tauri/src/chat/channel.rs`: Multiple `unwrap()` calls on message sending.

### 2.2. Security: Missing Content Security Policy (CSP)
In `src-tauri/tauri.conf.json`, the CSP is explicitly set to `null`.
```json
"security": {
  "csp": null
}
```
**Risk**: This allows Cross-Site Scripting (XSS). If an AI model or a malicious peer injects a script tag into a message, it will execute in the context of the application, potentially stealing keys or accessing local files.

### 2.3. Concurrency: Mutex Poisoning Risk
The code frequently uses `.lock().unwrap()` (e.g., `src-tauri/src/chat/rate_limit.rs`).
**Risk**: If a thread panics while holding a lock (which is likely given the `unwrap()` usage elsewhere), the lock becomes "poisoned". Subsequent threads trying to access that lock will also panic, causing a cascading failure of the entire backend.

---

## 3. Moderate Issues (Should Fix)

### 3.1. Frontend Type Safety
In `src/api.ts`, type safety is bypassed using `any` casting:
```typescript
return typeof result === 'object' && 'response' in result ? (result as any).response : result;
```
**Impact**: This defeats the purpose of TypeScript and can lead to runtime errors in the frontend if the backend API response structure changes.

### 3.2. MCP Server Security
The MCP server (`src-tauri/src/mcp.rs`) binds to `127.0.0.1` but appears to lack authentication.
**Impact**: Any malicious process running on the user's machine can connect to port 9001 and control the Council (ask questions, list sessions) without permission.

### 3.3. Lack of Global Panic Handler
`src-tauri/src/main.rs` does not implement a global panic hook.
**Impact**: When the application crashes (due to the `unwrap()` issues), it will likely close silently without logging the cause to a file, making debugging extremely difficult for end-users.

---

## 4. Suggestions & Recommendations

1.  **Eradicate `unwrap()`**: Replace all instances of `unwrap()` and `expect()` with proper `match` statements or `?` operator propagation. Return `Result<T, String>` to the frontend and handle errors gracefully in the UI.
2.  **Implement CSP**: Set a strict CSP in `tauri.conf.json`.
    ```json
    "csp": "default-src 'self'; img-src 'self' asset: https://asset.localhost; style-src 'self' 'unsafe-inline';"
    ```
3.  **Sanitize Inputs**: Ensure all inputs from the P2P network and MCP server are sanitized before processing or rendering.
4.  **Structured Logging**: Implement `log::set_boxed_logger` or `tracing` with a file appender to capture panic logs.
5.  **Frontend Error Boundaries**: Wrap main UI components in error boundaries to prevent the white screen of death if a component fails.

---

## 5. Strengths

*   **Architecture**: The separation of concerns between `CouncilSessionManager`, `P2PNetwork`, and `Ollama` integration is clean and logical.
*   **Cryptography**: Correct usage of `ed25519-dalek` for signing and verification.
*   **Networking**: `libp2p` configuration with `gossipsub` and `noise` encryption is industry-standard for this type of application.
*   **Testing**: The project has a good number of tests (103 passing), which provides a safety net for refactoring.

## 6. Questions for the Team

1.  Is there a plan to implement a "Safe Mode" that disables P2P features if the network is detected as hostile?
2.  How are agent system prompts validated to prevent "jailbreak" attempts via the P2P network?
3.  What is the strategy for database migration (SQLite) as the schema evolves?

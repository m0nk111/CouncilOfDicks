# Lessons Learned - Council Of Dicks Development

## 2025-12-04: Web Mode & Configuration Persistence

### 1. Configuration Persistence in API
**Issue:** The user handle was saving to disk but resetting on frontend reload.
**Root Cause:** The `GET /api/config` endpoint was returning a partial JSON object that excluded the `user_handle` field. The frontend, receiving `undefined`, fell back to the default "human_user".
**Lesson:** Always verify that the "Read" endpoint matches the schema of the "Write" endpoint. Persistence requires both saving correctly AND loading correctly.
**Fix:** Updated `get_config` in `web_server.rs` to include all relevant config fields.

### 2. Hardcoded Prompts vs. Configuration
**Issue:** The prompt for generating random questions was hardcoded in Rust, making it impossible to tweak without recompiling.
**Lesson:** Any text that might need tuning (prompts, system messages, error templates) should be externalized to the configuration file.
**Fix:** Moved the prompt to `config/app_config.json` under `question_generation_prompt`.

### 3. Chat Bot "Round Robin" Logic
**Issue:** Users reported that agents weren't responding to general messages (without mentions).
**Root Cause:** The `queue_round_robin_response` call was commented out in `chat_bot.rs` during previous testing to reduce noise, but never re-enabled.
**Lesson:** "Temporary" hacks for testing (like commenting out logic) must be tracked or marked with `TODO` to ensure they are reverted before release.
**Fix:** Re-enabled the round-robin logic and added debug logging to trace agent selection.

### 4. Web Mode vs. Tauri Mode
**Issue:** The frontend in "Web Mode" (browser) couldn't communicate with the backend because it was trying to use Tauri's IPC instead of HTTP.
**Lesson:** When building a hybrid Tauri/Web app, the API adapter layer is critical. It must robustly detect the environment and switch strategies (IPC vs. HTTP fetch).
**Fix:** Updated `api-adapter.ts` to correctly point to port 8080 when running in the browser.

## 📚 Related Documentation

- **[ROADMAP.md](ROADMAP.md)**: How these lessons influence future development.
- **[HEADLESS.md](HEADLESS.md)**: Lessons learned from headless deployment challenges.

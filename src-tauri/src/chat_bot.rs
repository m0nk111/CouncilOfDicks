use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use serde::{Deserialize, Serialize};

use crate::{
    agents::{Agent, AgentPool},
    chat::{AuthorType, ChannelType, Message},
    provider_dispatch, prompt, AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatBotStatus {
    pub queue: VecDeque<String>,
    pub current_thinking: Option<String>,
    pub current_reasoning: Option<String>,
}

#[derive(Debug, Clone)]
struct PendingResponse {
    agent: Agent,
    message: Message,
    stage_directive: Option<&'static str>,
    force_response: bool,
}

pub struct ChatBot {
    app_state: Arc<AppState>,
    agent_pool: Arc<AgentPool>,
    last_message_id: Option<String>,
    ai_to_ai_remaining: u32,
    warned_infinite_loop: bool,
    thread_origin_id: Option<String>,
    finalize_after_queue_empty: bool,
    thread_finalized: bool,
    enabled: bool,
    max_agents_per_message: usize,
    next_agent_index: usize,
    pending_responses: VecDeque<PendingResponse>,
}

impl ChatBot {
    pub fn new(app_state: Arc<AppState>, agent_pool: Arc<AgentPool>) -> Self {
        Self {
            app_state,
            agent_pool,
            last_message_id: None,
            ai_to_ai_remaining: 0,
            warned_infinite_loop: false,
            thread_origin_id: None,
            finalize_after_queue_empty: false,
            thread_finalized: false,
            enabled: true,
            max_agents_per_message: 4,
            next_agent_index: 0,
            pending_responses: VecDeque::new(),
        }
    }

    /// Start monitoring #general channel for new messages
    pub async fn start_monitoring(&mut self) {
        self.app_state
            .log_info("chat_bot", "🤖 Starting chat bot - monitoring #general");

        loop {
            if !self.enabled {
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            // Check for new messages every 1 second (faster for queue processing)
            sleep(Duration::from_secs(1)).await;

            if let Err(e) = self.tick().await {
                self.app_state
                    .log_error("chat_bot", &format!("Error: {}", e));
            }
        }
    }

    async fn tick(&mut self) -> Result<(), String> {
        // 1. Check for new messages and populate queue
        self.check_for_new_messages().await?;

        // 2. Process queue
        self.process_queue().await?;

        // 3. If a debate thread exhausted its AI-to-AI budget, finalize it once all queued
        // responses have been posted.
        self.maybe_finalize_thread().await;

        Ok(())
    }

    async fn maybe_finalize_thread(&mut self) {
        if !self.finalize_after_queue_empty || self.thread_finalized {
            return;
        }

        // Only finalize once all queued responses are done.
        if !self.pending_responses.is_empty() {
            return;
        }
        {
            let status = self.app_state.chat_bot_status.lock().unwrap();
            if status.current_thinking.is_some() {
                return;
            }
        }

        let origin_id = match self.thread_origin_id.clone() {
            Some(v) => v,
            None => {
                self.app_state
                    .log_warn("chat_bot", "⚠️ Cannot finalize thread: missing origin id");
                self.finalize_after_queue_empty = false;
                self.thread_finalized = true;
                return;
            }
        };

        self.finalize_after_queue_empty = false;
        self.thread_finalized = true;

        self.app_state.log_info(
            "chat_bot",
            "🧵 Finalizing #general debate thread → summary + topic enqueue",
        );

        let app_state = self.app_state.clone();
        let agent_pool = self.agent_pool.clone();
        let app_state_for_log = app_state.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::finalize_thread_task(app_state, agent_pool, origin_id).await {
                app_state_for_log
                    .log_warn("chat_bot", &format!("⚠️ Thread finalization failed: {}", e));
            }
        });
    }

    async fn finalize_thread_task(
        app_state: Arc<AppState>,
        agent_pool: Arc<AgentPool>,
        origin_id: String,
    ) -> Result<(), String> {
        #[derive(Debug, Deserialize)]
        struct ThreadSummary {
            summary: String,
            topic: String,
        }

        // Fetch a larger window of messages so we can slice the thread.
        let messages = app_state
            .channel_manager
            .get_messages(ChannelType::General, 200, 0)
            .map_err(|e| format!("failed to read messages: {}", e))?;
        if messages.is_empty() {
            return Err("no messages available to summarize".to_string());
        }

        // Messages are newest-first.
        let thread_messages: Vec<Message> = if let Some(idx) = messages.iter().position(|m| m.id == origin_id) {
            let slice = &messages[..=idx];
            slice
                .iter()
                .rev()
                .filter(|m| m.author_type != AuthorType::System)
                .cloned()
                .collect()
        } else {
            // If the origin fell out of history, take a reasonable tail.
            messages
                .iter()
                .take(40)
                .rev()
                .filter(|m| m.author_type != AuthorType::System)
                .cloned()
                .collect()
        };

        if thread_messages.len() < 2 {
            return Err("thread too small to summarize".to_string());
        }

        // Build a compact transcript.
        let mut transcript = String::new();
        for m in thread_messages.iter().rev().take(40).rev() {
            let mut line = m.content.clone();
            if line.len() > 600 {
                line.truncate(600);
                line.push_str("…");
            }
            transcript.push_str(&format!("{}: {}\n", m.author, line));
        }

        // Pick a small/fast active agent as summarizer.
        let mut agents = agent_pool.list_agents().await;
        agents.retain(|a| a.active);
        if agents.is_empty() {
            return Err("no active agents available to summarize".to_string());
        }
        agents.sort_by_key(|a| ChatBot::model_size_bucket(&a.model));
        let summarizer = agents
            .into_iter()
            .next()
            .ok_or_else(|| "no summarizer agent".to_string())?;

        app_state.log_debug(
            "chat_bot",
            &format!(
                "🔍 Summarizer selected: {} ({}:{})",
                summarizer.name, summarizer.provider, summarizer.model
            ),
        );

        let config = app_state.get_config();
        let timeout_secs = Some(summarizer.timeout_secs.unwrap_or(45).min(45));

        let prompt = format!(
            r#"You are the Council Of Dicks thread summarizer.

Given this #general transcript, produce a concise summary and a single derived topic.

Return ONLY valid JSON in this exact shape:
{{"summary":"...","topic":"..."}}

Rules:
- summary: 3-8 bullet points, plain text (use hyphen '-' lines), no markdown headers
- topic: <= 80 characters, plain text, no quotes around it, no emojis
- Do not include anything except the JSON

Transcript:
{}"#,
            transcript
        );

        let raw = provider_dispatch::generate_with_timeout(
            &summarizer.provider,
            &summarizer.model,
            prompt,
            None,
            &config,
            Some(app_state.logger.clone()),
            timeout_secs,
        )
        .await?;

        // Try strict JSON parse, otherwise attempt to extract the first {...} block.
        let parsed: ThreadSummary = match serde_json::from_str(raw.trim()) {
            Ok(v) => v,
            Err(_) => {
                let s = raw.trim();
                let start = s.find('{');
                let end = s.rfind('}');
                if let (Some(a), Some(b)) = (start, end) {
                    serde_json::from_str::<ThreadSummary>(&s[a..=b])
                        .map_err(|e| format!("failed to parse summarizer JSON: {}", e))?
                } else {
                    return Err("summarizer did not return JSON".to_string());
                }
            }
        };

        let summary = parsed.summary.trim().to_string();
        let mut topic = parsed.topic.trim().to_string();
        if topic.len() > 80 {
            topic.truncate(80);
        }
        if topic.is_empty() {
            return Err("empty topic from summarizer".to_string());
        }

        // Announce summary in #general.
        let summary_msg = Message::new(
            ChannelType::General,
            "System".to_string(),
            AuthorType::System,
            format!("🧵 Thread summary (auto)\n\n{}", summary),
        );
        let _ = app_state.channel_manager.send_message(summary_msg);

        // Enqueue topic and announce in #topic.
        app_state
            .topic_manager
            .enqueue_topic_with_announcement(&app_state, topic.clone())
            .map_err(|e| format!("failed to enqueue topic: {}", e))?;

        let status = app_state.topic_manager.get_status();
        let mut queue_text = String::new();
        if status.queued_topics.is_empty() {
            queue_text.push_str("(queue is empty)");
        } else {
            for (i, t) in status.queued_topics.iter().enumerate() {
                queue_text.push_str(&format!("{}. {}\n", i + 1, t));
            }
        }

        let topic_msg = Message::new(
            ChannelType::Topic,
            "System".to_string(),
            AuthorType::System,
            format!(
                "📌 Auto-derived topic enqueued from #general\n\nTopic: {}\n\nSummary:\n{}\n\n📥 Queue:\n{}",
                topic, summary, queue_text
            ),
        );
        let _ = app_state.channel_manager.send_message(topic_msg);

        app_state.log_success(
            "chat_bot",
            &format!("✅ Auto-topic enqueued: '{}'", topic),
        );

        Ok(())
    }

    async fn check_for_new_messages(&mut self) -> Result<(), String> {
        // Get recent messages from #general (newest first)
        let messages = self
            .app_state
            .channel_manager
            .get_messages(ChannelType::General, 20, 0)?;

        if messages.is_empty() {
            return Ok(());
        }

        // Process all messages newer than last_message_id (oldest -> newest)
        let new_messages: Vec<Message> = match self.last_message_id.as_ref() {
            None => vec![messages[0].clone()],
            Some(last_id) => {
                if let Some(idx) = messages.iter().position(|m| &m.id == last_id) {
                    let slice = &messages[..idx];
                    slice.iter().rev().cloned().collect()
                } else {
                    // If we can't find the last id (trimmed history), fall back to latest only.
                    vec![messages[0].clone()]
                }
            }
        };

        if new_messages.is_empty() {
            return Ok(());
        }

        for msg in new_messages {
            // Update last processed message
            self.last_message_id = Some(msg.id.clone());

            let config = self.app_state.get_config();

            let is_human = msg.author_type == AuthorType::Human;
            let is_ai = msg.author_type == AuthorType::AI;

            // Human messages always trigger; AI messages trigger only if enabled and budget remains.
            if is_human {
                // Reset AI-to-AI budget on each human message to enable debate, but prevent infinite loops.
                self.thread_origin_id = Some(msg.id.clone());
                self.finalize_after_queue_empty = false;
                self.thread_finalized = false;
                if config.chat_ai_to_ai_infinite_loop {
                    if !self.warned_infinite_loop {
                        self.app_state.log_warn(
                            "chat_bot",
                            "⚠️ chat_ai_to_ai_infinite_loop=true: AI-to-AI loop protection disabled (may cause infinite loops)",
                        );
                        self.warned_infinite_loop = true;
                    }
                    self.ai_to_ai_remaining = u32::MAX;
                } else {
                    self.ai_to_ai_remaining = config.chat_ai_to_ai_budget;
                }
            } else if is_ai {
                if !self.pending_responses.is_empty() {
                    self.app_state.log_debug(
                        "chat_bot",
                        "🧵 Structured queue in progress; deferring AI-triggered follow-up",
                    );
                    continue;
                }
                if !config.chat_allow_ai_to_ai {
                    continue;
                }
                if !config.chat_ai_to_ai_infinite_loop {
                    if self.ai_to_ai_remaining == 0 {
                        self.app_state.log_debug(
                            "chat_bot",
                            "🧯 AI-to-AI budget exhausted; ignoring AI message",
                        );
                        continue;
                    }
                    self.ai_to_ai_remaining = self.ai_to_ai_remaining.saturating_sub(1);

                    // If we just consumed the last budget unit, finalize after the current
                    // response queue drains (so the summary includes the last batch of replies).
                    if self.ai_to_ai_remaining == 0 && !self.thread_finalized {
                        self.finalize_after_queue_empty = true;
                        self.app_state.log_info(
                            "chat_bot",
                            "🧾 AI-to-AI budget reached 0 → will auto-summarize when queue drains",
                        );
                    }
                }
            } else {
                // System messages do not trigger
                continue;
            }

            self.app_state.log_debug(
                "chat_bot",
                &format!(
                    "📨 Trigger message ({}): {}: {}",
                    match msg.author_type {
                        AuthorType::Human => "human",
                        AuthorType::AI => "ai",
                        AuthorType::System => "system",
                    },
                    msg.author,
                    msg.content
                ),
            );

            // Check for @mentions
            let agents = self.agent_pool.list_agents().await;
            let mut mentioned_agents = Vec::new();
            let mut seen_agent_ids = HashSet::new();
            let message_lower = msg.content.to_lowercase();
            let mention_all = msg.channel == ChannelType::General
                && (message_lower.contains("@all") || message_lower.contains("@everyone"));

            if mention_all {
                self.app_state
                    .log_info("chat_bot", "📣 Broadcast mention detected: @all/@everyone");
            }

            for agent in &agents {
                // Never respond to your own message
                if agent.name == msg.author {
                    continue;
                }

                if mention_all && agent.active && seen_agent_ids.insert(agent.id.clone()) {
                    mentioned_agents.push(agent.clone());
                    continue;
                }

                let handle = format!("@{}", agent.handle);
                if message_lower.contains(&handle.to_lowercase())
                    && seen_agent_ids.insert(agent.id.clone())
                {
                    self.app_state
                        .log_debug("chat_bot", &format!("Found mention for handle: {}", handle));
                    mentioned_agents.push(agent.clone());
                }
            }

            if mention_all && is_human {
                self.queue_structured_debate(msg.clone()).await;
            } else if !mentioned_agents.is_empty() {
                self.app_state.log_info(
                    "chat_bot",
                    &format!("🎯 Direct mention detected for {} agents", mentioned_agents.len()),
                );
                for agent in mentioned_agents {
                    self.queue_response(agent, msg.clone()).await;
                }
            } else {
                // No mentions, use round robin
                self.queue_round_robin_response(msg.clone()).await;
            }
        }

        Ok(())
    }

    async fn queue_response(&mut self, agent: Agent, message: Message) {
        self.queue_response_with_plan(agent, message, None, false).await;
    }

    async fn queue_response_with_plan(
        &mut self,
        agent: Agent,
        message: Message,
        stage_directive: Option<&'static str>,
        force_response: bool,
    ) {
        if !agent.active {
            self.app_state.log_debug(
                "chat_bot",
                &format!("⏭️ Skipping inactive agent: {}", agent.name),
            );
            return;
        }

        let config = self.app_state.get_config();
        if !provider_dispatch::is_provider_configured(&agent.provider, &config) {
            self.app_state.log_warn(
                "chat_bot",
                &format!(
                    "⚠️ Skipping {}: provider '{}' is not configured",
                    agent.name, agent.provider
                ),
            );
            return;
        }

        self.app_state.log_debug(
            "chat_bot",
            &format!("➕ Queuing agent: {}", agent.name),
        );

        // Add to internal queue
        self.pending_responses.push_back(PendingResponse {
            agent: agent.clone(),
            message,
            stage_directive,
            force_response,
        });
        
        // Update public status
        let mut status = self.app_state.chat_bot_status.lock().unwrap();
        status.queue.push_back(agent.name);
    }

    async fn queue_structured_debate(&mut self, message: Message) {
        let config = self.app_state.get_config();
        let agents = self.agent_pool.list_agents().await;
        let available_agents: Vec<Agent> = agents
            .into_iter()
            .filter(|agent| agent.active)
            .filter(|agent| agent.name != message.author)
            .filter(|agent| provider_dispatch::is_provider_configured(&agent.provider, &config))
            .collect();

        if available_agents.is_empty() {
            self.app_state.log_warn(
                "chat_bot",
                "⚠️ No active, configured agents available for structured debate",
            );
            return;
        }

        const STAGES: [(&str, &str); 4] = [
            (
                "planner",
                "Role: Planner. First, frame the problem, identify the key constraints, and propose the best plan of attack before anyone else executes or critiques.",
            ),
            (
                "executor",
                "Role: Executor. Build on the current discussion, turn the plan into a concrete answer, and push the discussion toward an actionable conclusion.",
            ),
            (
                "reviewer",
                "Role: Reviewer. Critique the plan and execution so far, identify weak assumptions, and strengthen the emerging conclusion.",
            ),
            (
                "arbiter",
                "Role: Arbiter. Read the full discussion so far, resolve disagreements, and produce the clearest final conclusion for the thread.",
            ),
        ];

        let mut used_ids = HashSet::new();
        let mut selected = Vec::new();

        for (stage, directive) in STAGES {
            if let Some(agent) = available_agents
                .iter()
                .filter(|agent| !used_ids.contains(&agent.id))
                .max_by_key(|agent| self.stage_score(agent, stage))
                .cloned()
            {
                used_ids.insert(agent.id.clone());
                selected.push((agent, directive));
            }
        }

        if selected.is_empty() {
            self.app_state.log_warn(
                "chat_bot",
                "⚠️ Structured debate found no selectable agents; falling back to round robin",
            );
            self.queue_round_robin_response(message).await;
            return;
        }

        self.app_state.log_info(
            "chat_bot",
            &format!("🧠 Structured debate queued with {} stages", selected.len()),
        );

        for (agent, directive) in selected {
            self.queue_response_with_plan(agent, message.clone(), Some(directive), true)
                .await;
        }
    }

    async fn queue_round_robin_response(&mut self, message: Message) {
        self.app_state.log_debug("chat_bot", "🔄 Initiating round robin selection");
        
        let agents = self.agent_pool.list_agents().await;
        if agents.is_empty() {
            self.app_state.log_warn("chat_bot", "⚠️ No agents available for round robin");
            return;
        }

        let mut active_agents: Vec<Agent> = agents
            .into_iter()
            .filter(|a| a.active)
            .filter(|a| a.name != message.author)
            .filter(|a| provider_dispatch::is_provider_configured(&a.provider, &self.app_state.get_config()))
            .collect();
        if active_agents.is_empty() {
            self.app_state.log_warn("chat_bot", "⚠️ No active, configured agents found");
            return;
        }

        // Prefer smaller/faster models first to avoid one slow model (e.g., 30B/32B)
        // blocking all other queued responses.
        active_agents.sort_by_key(|a| Self::model_size_bucket(&a.model));

        self.app_state.log_debug("chat_bot", &format!("Found {} active agents", active_agents.len()));

        // Pick next agents (round robin)
        // For AI-triggered messages, respond with fewer agents to reduce ping-pong.
        let agents_to_select = if message.author_type == AuthorType::AI {
            2usize.min(self.max_agents_per_message)
        } else {
            self.max_agents_per_message
        };
        let mut selected_agents = Vec::new();
        for _ in 0..agents_to_select {
            if self.next_agent_index >= active_agents.len() {
                self.next_agent_index = 0;
            }
            let agent = active_agents[self.next_agent_index].clone();
            self.app_state.log_debug("chat_bot", &format!("Selected agent: {}", agent.name));
            selected_agents.push(agent);
            self.next_agent_index += 1;
        }

        for agent in selected_agents {
            self.queue_response(agent, message.clone()).await;
        }
    }

    fn model_size_bucket(model: &str) -> u32 {
        // Extract a rough "Nb" size hint from model strings like "gemma2:27b" or "qwen3-30b".
        // If absent (e.g., "gpt-4o"), assume mid-sized.
        let lower = model.to_lowercase();
        let bytes = lower.as_bytes();
        let mut best: Option<u32> = None;

        for i in 0..bytes.len() {
            if bytes[i] != b'b' {
                continue;
            }

            // Walk backwards to capture digits immediately preceding 'b'
            let mut j = i;
            while j > 0 && bytes[j - 1].is_ascii_digit() {
                j -= 1;
            }
            if j == i {
                continue;
            }

            if let Ok(n) = lower[j..i].parse::<u32>() {
                best = Some(best.map(|cur| cur.min(n)).unwrap_or(n));
            }
        }

        best.unwrap_or(15)
    }

    async fn process_queue(&mut self) -> Result<(), String> {
        // Check if already thinking
        {
            let status = self.app_state.chat_bot_status.lock().unwrap();
            if status.current_thinking.is_some() {
                return Ok(());
            }
        }

        // Pop next response
        if let Some(pending) = self.pending_responses.pop_front() {
            let agent = pending.agent;
            let msg = pending.message;
            let stage_directive = pending.stage_directive;
            let force_response = pending.force_response;
            // Update status
            {
                let mut status = self.app_state.chat_bot_status.lock().unwrap();
                status.queue.pop_front(); // Remove from public queue
                status.current_thinking = Some(agent.name.clone());
                status.current_reasoning = Some("Checking relevance...".to_string());
            }

            let config = self.app_state.get_config();
            let context = self.build_context_for_message(&msg).await;
            
            // First check if this agent has something relevant to add
            let should_respond = if force_response {
                true
            } else {
                self.should_respond(&agent, &msg, &context, &config).await
            };
            
            if !should_respond {
                self.app_state.log_info(
                    "chat_bot",
                    &format!("⏭️ {} has nothing to add, skipping", agent.name),
                );
                // Clear status and move on
                {
                    let mut status = self.app_state.chat_bot_status.lock().unwrap();
                    status.current_thinking = None;
                    status.current_reasoning = None;
                }
                return Ok(());
            }
            
            // Update status for actual response generation
            {
                let mut status = self.app_state.chat_bot_status.lock().unwrap();
                status.current_reasoning = Some("Formulating response...".to_string());
            }

            // Execute response
            if let Err(e) = self
                .respond_with_agent(&agent, &msg, &context, &config, stage_directive)
                .await
            {
                self.app_state.log_error("chat_bot", &format!("Agent {} error: {}", agent.name, e));
            }

            // Clear status
            {
                let mut status = self.app_state.chat_bot_status.lock().unwrap();
                status.current_thinking = None;
                status.current_reasoning = None;
            }
        }

        Ok(())
    }

    /// Ask the agent if they have something relevant to contribute
    async fn should_respond(
        &self,
        agent: &Agent,
        msg: &Message,
        context: &str,
        config: &crate::config::AppConfig,
    ) -> bool {
        // Dev/testing override: force 100% relevance (always respond)
        if config.chat_force_relevance {
            self.app_state.log_debug(
                "chat_bot",
                &format!(
                    "🔧 chat_force_relevance=true → forcing response for '{}'",
                    agent.name
                ),
            );
            return true;
        }

        // Heuristic: if the user asked a direct question, default to responding.
        // The relevance-gate is useful for chatter, but for questions it often suppresses debate.
        if msg.content.contains('?') {
            self.app_state.log_debug(
                "chat_bot",
                &format!(
                    "🐛 Bypassing relevance check for '{}' (question detected)",
                    agent.name
                ),
            );
            return true;
        }

        let check_prompt = format!(
            r#"You are {} - {}

Recent conversation:
{}

Latest message from {}: "{}"

TASK: Return a relevance score between 0.0 and 1.0 (inclusive) indicating how valuable it is for you to respond.

Rules:
- Output ONLY a single number (e.g. 0.0, 0.25, 0.73, 1.0)
- 1.0 = highly relevant and uniquely valuable
- 0.0 = not relevant / would be noise
"#,
            agent.name,
            agent.system_prompt.lines().next().unwrap_or("An AI assistant"),
            if context.is_empty() { "(no context)" } else { context },
            msg.author,
            msg.content
        );

        // Relevance checks must be fast; cap timeout so one slow model can't stall the queue.
        // Response generation still uses the agent's configured timeout.
        let relevance_timeout_secs = Some(agent.timeout_secs.unwrap_or(15).min(15));

        // Use a smaller/faster model for the check if available, otherwise use agent's model
        let check_model = &agent.model;
        
        match provider_dispatch::generate_with_timeout(
            &agent.provider,
            check_model,
            check_prompt,
            None,
            config,
            Some(self.app_state.logger.clone()),
            relevance_timeout_secs,
        )
        .await
        {
            Ok(response) => {
                let trimmed = response.trim();
                let token = trimmed.split_whitespace().next().unwrap_or("");

                let score: f32 = match token.parse::<f32>() {
                    Ok(v) => v.clamp(0.0, 1.0),
                    Err(_) => {
                        // If the model ignores instructions, default to responding.
                        self.app_state.log_warn(
                            "chat_bot",
                            &format!(
                                "⚠️ Non-numeric relevance response for {}: '{}' (defaulting to 1.0)",
                                agent.name,
                                &trimmed[..trimmed.len().min(120)]
                            ),
                        );
                        1.0
                    }
                };

                let threshold = config.chat_relevance_threshold.clamp(0.0, 1.0);
                let should = score >= threshold;
                self.app_state.log_debug(
                    "chat_bot",
                    &format!(
                        "📊 {} relevance score: {:.2} (threshold {:.2}) → {}",
                        agent.name,
                        score,
                        threshold,
                        if should { "will respond" } else { "skipping" }
                    ),
                );
                should
            }
            Err(e) => {
                self.app_state.log_warn("chat_bot", &format!("⚠️ Relevance check failed for {}: {}", agent.name, e));
                false
            }
        }
    }

    async fn respond_with_agent(
        &self,
        agent: &Agent,
        msg: &Message,
        context: &str,
        config: &crate::config::AppConfig,
        stage_directive: Option<&'static str>,
    ) -> Result<(), String> {
        let system_prompt = prompt::compose_system_prompt(&agent.system_prompt);
        let stage_section = stage_directive
            .map(|directive| format!("# Deliberation Role\n{}\n\n", directive))
            .unwrap_or_default();
        let prompt = if context.is_empty() {
            format!(
                "{}Latest human message from {}:\n{}\n\nRespond as {}. Start your response by mentioning the participants you are addressing (e.g. @human_user, @technical_architect). Keep it concise and pragmatic.",
                stage_section,
                msg.author,
                msg.content,
                agent.name
            )
        } else {
            format!(
                "{}# Recent Conversation\n{}\n\n# Latest human message from {}\n{}\n\nRespond as {}. Start your response by mentioning the participants you are addressing (e.g. @human_user, @technical_architect). Keep it concise and pragmatic, grounded in the above context.",
                stage_section,
                context,
                msg.author,
                msg.content,
                agent.name
            )
        };

        self.app_state.log_network(
            "chat_bot",
            &format!("→ {}:{}", agent.provider, agent.model),
        );

        let start_time = std::time::Instant::now();
        let context_size = prompt.len() + system_prompt.len();

        match provider_dispatch::generate_with_timeout(
            &agent.provider,
            &agent.model,
            prompt,
            Some(system_prompt),
            config,
            Some(self.app_state.logger.clone()),
            agent.timeout_secs,
        )
        .await
        {
            Ok(response) => {
                let elapsed_ms = start_time.elapsed().as_secs_f64() * 1000.0;
                
                // Estimate tokens (roughly 4 chars per token)
                let input_tokens = (context_size / 4) as u64;
                let output_tokens = (response.len() / 4) as u64;
                
                // Record stats for this agent
                self.app_state.agent_pool.record_success(
                    &agent.id,
                    input_tokens,
                    output_tokens,
                    elapsed_ms,
                    context_size,
                ).await;

                if response.trim().is_empty() {
                    self.app_state.log_error("chat_bot", "❌ Received empty response from provider");
                    return Err("Empty response from provider".to_string());
                }

                self.app_state
                    .log_success("chat_bot", &format!("← Response: {} chars in {:.0}ms", response.len(), elapsed_ms));

                let reply = Message::new(
                    ChannelType::General,
                    agent.name.clone(),
                    AuthorType::AI,
                    response,
                );

                match self.app_state.channel_manager.send_message(reply.clone()) {
                    Ok(_) => {
                        self.app_state
                            .log_success("chat_bot", "✅ Response sent to #general");
                        let _ = self.app_state.websocket_broadcast.send(reply);
                    }
                    Err(e) => {
                        self.app_state
                            .log_error("chat_bot", &format!("Failed to send response: {}", e));
                    }
                }
                Ok(())
            }
            Err(e) => {
                let elapsed_ms = start_time.elapsed().as_secs_f64() * 1000.0;
                self.app_state.agent_pool.record_failure(&agent.id, elapsed_ms).await;
                Err(e)
            }
        }
    }

    async fn build_context_for_message(&self, message: &Message) -> String {
        if message.channel == ChannelType::Knowledge {
            if let Some(kb) = &self.app_state.knowledge_bank {
                self.app_state
                    .log_debug("chat_bot", "🔍 Searching Knowledge Bank for consensus");
                match kb.semantic_search(&message.content, 3).await {
                    Ok(results) => {
                        if results.is_empty() {
                            "No relevant past decisions found.".to_string()
                        } else {
                            let mut ctx = String::from("### Relevant Past Decisions (Consensus):\n\n");
                            for result in results {
                                ctx.push_str(&format!(
                                    "- **Question:** {}\n  **Verdict:** {}\n\n",
                                    result.question, result.text_snippet
                                ));
                            }
                            ctx
                        }
                    }
                    Err(_) => "Error retrieving knowledge.".to_string(),
                }
            } else {
                "Knowledge bank disabled.".to_string()
            }
        } else {
            let mut ctx = String::new();
            let recent_messages = self
                .app_state
                .channel_manager
                .get_messages(message.channel, 10, 0)
                .unwrap_or_default();

            ctx.push_str(&self.build_context(&recent_messages));

            if let Some(kb) = &self.app_state.knowledge_bank {
                if let Ok(rag_results) = kb.search_channel_context(message.channel, &message.content, 3).await {
                    if !rag_results.is_empty() {
                        ctx.push_str("\n\n### Relevant Context from this discussion:\n");
                        for msg in rag_results {
                            ctx.push_str(&format!("- {}\n", msg));
                        }
                    }
                }
            }

            ctx
        }
    }

    fn stage_score(&self, agent: &Agent, stage: &str) -> i32 {
        let role = agent.metadata.get("role").cloned().unwrap_or_default().to_lowercase();
        let combined = format!(
            "{} {} {}",
            agent.name.to_lowercase(),
            role,
            agent.system_prompt.to_lowercase()
        );

        let keywords: &[&str] = match stage {
            "planner" => &["planner", "strategist", "architect", "reasoner", "oracle"],
            "executor" => &["executor", "builder", "coder", "implementer", "pragmatist"],
            "reviewer" => &["reviewer", "skeptic", "critic", "analyst", "veritas"],
            "arbiter" => &["arbiter", "mediator", "judge", "moderator", "whisper"],
            _ => &[],
        };

        let mut score = 0;
        for keyword in keywords {
            if combined.contains(keyword) {
                score += 10;
            }
        }

        score += (40 - Self::model_size_bucket(&agent.model) as i32).max(0);
        score
    }

    fn build_context(&self, messages: &[Message]) -> String {
        messages
            .iter()
            .rev()
            .map(|msg| format!("{}: {}", msg.author, msg.content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        self.app_state.log_info("chat_bot", "✅ Chat bot enabled");
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.app_state.log_info("chat_bot", "⏸️ Chat bot disabled");
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

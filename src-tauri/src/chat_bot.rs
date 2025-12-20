use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use serde::{Deserialize, Serialize};

use crate::{
    agents::{Agent, AgentPool},
    chat::{AuthorType, ChannelType, Message},
    ollama, prompt, AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatBotStatus {
    pub queue: VecDeque<String>,
    pub current_thinking: Option<String>,
    pub current_reasoning: Option<String>,
}

pub struct ChatBot {
    app_state: Arc<AppState>,
    agent_pool: Arc<AgentPool>,
    last_message_id: Option<String>,
    enabled: bool,
    max_agents_per_message: usize,
    next_agent_index: usize,
    pending_responses: VecDeque<(Agent, Message, String)>,
}

impl ChatBot {
    pub fn new(app_state: Arc<AppState>, agent_pool: Arc<AgentPool>) -> Self {
        Self {
            app_state,
            agent_pool,
            last_message_id: None,
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

        Ok(())
    }

    async fn check_for_new_messages(&mut self) -> Result<(), String> {
        // Get recent messages from #general
        let messages = self
            .app_state
            .channel_manager
            .get_messages(ChannelType::General, 10, 0)?;

        if messages.is_empty() {
            return Ok(());
        }

        // Only check the very latest message to avoid oscillation
        let latest_msg = &messages[0];

        // If it's a human message and we haven't processed it yet
        if latest_msg.author_type == AuthorType::Human 
            && Some(&latest_msg.id) != self.last_message_id.as_ref() 
        {
            self.app_state.log_debug(
                "chat_bot",
                &format!("📨 New message from {}: {}", latest_msg.author, latest_msg.content),
            );

            // Update last processed message
            self.last_message_id = Some(latest_msg.id.clone());

            // Check for @mentions
            let agents = self.agent_pool.list_agents().await;
            let mut mentioned_agents = Vec::new();

            for agent in &agents {
                let handle = format!("@{}", agent.handle);
                if latest_msg.content.to_lowercase().contains(&handle.to_lowercase()) {
                    self.app_state.log_debug("chat_bot", &format!("Found mention for handle: {}", handle));
                    mentioned_agents.push(agent.clone());
                }
            }

            if !mentioned_agents.is_empty() {
                self.app_state.log_info(
                    "chat_bot",
                    &format!("🎯 Direct mention detected for {} agents", mentioned_agents.len()),
                );
                for agent in mentioned_agents {
                    self.queue_response(agent, latest_msg.clone()).await;
                }
            } else {
                // No mentions, use round robin
                self.queue_round_robin_response(latest_msg.clone()).await;
            }
        }

        Ok(())
    }

    async fn queue_response(&mut self, agent: Agent, message: Message) {
        // Build context based on channel type
        let context = if message.channel == ChannelType::Knowledge {
            // For #knowledge, ONLY use Consensus results (Global Knowledge)
            if let Some(kb) = &self.app_state.knowledge_bank {
                self.app_state.log_debug("chat_bot", "🔍 Searching Knowledge Bank for consensus");
                match kb.semantic_search(&message.content, 3).await {
                    Ok(results) => {
                        if results.is_empty() {
                            "No relevant past decisions found.".to_string()
                        } else {
                            let mut ctx = String::from("### Relevant Past Decisions (Consensus):\n\n");
                            for result in results {
                                ctx.push_str(&format!("- **Question:** {}\n  **Verdict:** {}\n\n", 
                                    result.question, result.text_snippet));
                            }
                            ctx
                        }
                    }
                    Err(_) => "Error retrieving knowledge.".to_string()
                }
            } else {
                "Knowledge bank disabled.".to_string()
            }
        } else {
            // For #general, #topic, #vote: Use Channel-Scoped RAG + Recent History
            let mut ctx = String::new();
            
            // 1. Get recent messages (Short-term memory)
            let recent_messages = self
                .app_state
                .channel_manager
                .get_messages(message.channel, 10, 0)
                .unwrap_or_default();
            
            ctx.push_str(&self.build_context(&recent_messages));

            // 2. Get relevant older messages from this channel (Long-term channel memory)
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
        };

        self.app_state.log_debug(
            "chat_bot",
            &format!("➕ Queuing agent: {}", agent.name),
        );

        // Add to internal queue
        self.pending_responses.push_back((agent.clone(), message, context));
        
        // Update public status
        let mut status = self.app_state.chat_bot_status.lock().unwrap();
        status.queue.push_back(agent.name);
    }

    async fn queue_round_robin_response(&mut self, message: Message) {
        self.app_state.log_debug("chat_bot", "🔄 Initiating round robin selection");
        
        let agents = self.agent_pool.list_agents().await;
        if agents.is_empty() {
            self.app_state.log_warn("chat_bot", "⚠️ No agents available for round robin");
            return;
        }

        let active_agents: Vec<Agent> = agents.into_iter().filter(|a| a.active).collect();
        if active_agents.is_empty() {
            self.app_state.log_warn("chat_bot", "⚠️ No active agents found");
            return;
        }

        self.app_state.log_debug("chat_bot", &format!("Found {} active agents", active_agents.len()));

        // Pick next agents (round robin)
        let mut selected_agents = Vec::new();
        for _ in 0..self.max_agents_per_message {
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

    async fn process_queue(&mut self) -> Result<(), String> {
        // Check if already thinking
        {
            let status = self.app_state.chat_bot_status.lock().unwrap();
            if status.current_thinking.is_some() {
                return Ok(());
            }
        }

        // Pop next response
        if let Some((agent, msg, context)) = self.pending_responses.pop_front() {
            // Update status
            {
                let mut status = self.app_state.chat_bot_status.lock().unwrap();
                status.queue.pop_front(); // Remove from public queue
                status.current_thinking = Some(agent.name.clone());
                status.current_reasoning = Some("Checking relevance...".to_string());
            }

            let config = self.app_state.get_config();
            
            // First check if this agent has something relevant to add
            let should_respond = self.should_respond(&agent, &msg, &context, &config).await;
            
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
            if let Err(e) = self.respond_with_agent(&agent, &msg, &context, &config).await {
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
        let check_prompt = format!(
            r#"You are {} - {}

Recent conversation:
{}

Latest message from {}: "{}"

QUESTION: Do you have a unique, valuable perspective to add to this discussion that hasn't been covered yet?

Answer ONLY "YES" or "NO" (nothing else).
- YES = You have a distinct insight, counterpoint, or expertise to share
- NO = The topic is outside your expertise, already well-covered, or you'd just be repeating others"#,
            agent.name,
            agent.system_prompt.lines().next().unwrap_or("An AI assistant"),
            if context.is_empty() { "(no context)" } else { context },
            msg.author,
            msg.content
        );

        let auth = if let (Some(u), Some(p)) = (&config.ollama_username, &config.ollama_password) {
            Some((u.as_str(), p.as_str()))
        } else {
            None
        };

        // Use a smaller/faster model for the check if available, otherwise use agent's model
        let check_model = &agent.model;
        
        match ollama::ask_ollama_with_auth(
            &config.ollama_url,
            check_model,
            check_prompt,
            None,
            auth,
        )
        .await
        {
            Ok(response) => {
                let answer = response.trim().to_uppercase();
                let should = answer.starts_with("YES");
                self.app_state.log_debug(
                    "chat_bot",
                    &format!("🤔 {} relevance check: {} → {}", agent.name, answer, if should { "will respond" } else { "skipping" }),
                );
                should
            }
            Err(e) => {
                self.app_state.log_warn("chat_bot", &format!("⚠️ Relevance check failed for {}: {}", agent.name, e));
                true // Default to responding if check fails
            }
        }
    }

    async fn respond_with_agent(
        &self,
        agent: &Agent,
        msg: &Message,
        context: &str,
        config: &crate::config::AppConfig,
    ) -> Result<(), String> {
        let system_prompt = prompt::compose_system_prompt(&agent.system_prompt);
        let prompt = if context.is_empty() {
            format!(
                "Latest human message from {}:\n{}\n\nRespond as {}. Start your response by mentioning the participants you are addressing (e.g. @human_user, @technical_architect). Keep it concise and pragmatic.",
                msg.author,
                msg.content,
                agent.name
            )
        } else {
            format!(
                "# Recent Conversation\n{}\n\n# Latest human message from {}\n{}\n\nRespond as {}. Start your response by mentioning the participants you are addressing (e.g. @human_user, @technical_architect). Keep it concise and pragmatic, grounded in the above context.",
                context,
                msg.author,
                msg.content,
                agent.name
            )
        };

        self.app_state.log_network(
            "chat_bot",
            &format!("→ {}:{}", config.ollama_url, agent.model),
        );

        let auth = if let (Some(u), Some(p)) = (&config.ollama_username, &config.ollama_password) {
            Some((u.as_str(), p.as_str()))
        } else {
            None
        };

        match ollama::ask_ollama_with_auth(
            &config.ollama_url,
            &agent.model,
            prompt,
            Some(system_prompt),
            auth,
        )
        .await
        {
            Ok(response) => {
                if response.trim().is_empty() {
                    self.app_state.log_error("chat_bot", "❌ Received empty response from Ollama");
                    return Err("Empty response from Ollama".to_string());
                }

                self.app_state
                    .log_success("chat_bot", &format!("← Response: {} chars", response.len()));

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
            Err(e) => Err(e),
        }
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

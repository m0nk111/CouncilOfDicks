use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::{
    agents::{Agent, AgentPool},
    chat::{AuthorType, ChannelType, Message},
    ollama, prompt, AppState,
};

pub struct ChatBot {
    app_state: Arc<AppState>,
    agent_pool: Arc<AgentPool>,
    last_message_id: Option<String>,
    enabled: bool,
    max_agents_per_message: usize,
    next_agent_index: usize,
}

impl ChatBot {
    pub fn new(app_state: Arc<AppState>, agent_pool: Arc<AgentPool>) -> Self {
        Self {
            app_state,
            agent_pool,
            last_message_id: None,
            enabled: true,
            max_agents_per_message: 2,
            next_agent_index: 0,
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

            // Check for new messages every 2 seconds
            sleep(Duration::from_secs(2)).await;

            println!("[CHATBOT] Tick - checking for messages...");
            if let Err(e) = self.check_and_respond().await {
                self.app_state
                    .log_error("chat_bot", &format!("Error: {}", e));
                println!("[CHATBOT] Error: {}", e);
            }
        }
    }

    async fn check_and_respond(&mut self) -> Result<(), String> {
        // Get recent messages from #general
        let messages = self
            .app_state
            .channel_manager
            .get_messages(ChannelType::General, 10, 0)?;

        self.app_state.log_debug(
            "chat_bot",
            &format!("🔍 Checking messages: {} found", messages.len()),
        );

        if messages.is_empty() {
            return Ok(());
        }

        // Find first unprocessed human message
        let new_message = messages.iter().find(|msg| {
            let is_new = msg.author_type == AuthorType::Human
                && Some(&msg.id) != self.last_message_id.as_ref();
            if is_new {
                self.app_state.log_debug(
                    "chat_bot",
                    &format!("✨ Found new message: {} from {}", msg.id, msg.author),
                );
            }
            is_new
        });

        if let Some(msg) = new_message {
            self.app_state.log_debug(
                "chat_bot",
                &format!("📨 New message from {}: {}", msg.author, msg.content),
            );

            // Update last processed message
            self.last_message_id = Some(msg.id.clone());

            // Get available agents to respond
            let agents = self.agent_pool.list_agents().await;
            if agents.is_empty() {
                self.app_state
                    .log_debug("chat_bot", "⚠️ No agents available to respond");
                return Ok(());
            }

            // Build context from recent messages
            let recent_slice = &messages[..5.min(messages.len())];
            let context = self.build_context(recent_slice);
            let config = self.app_state.get_config();

            let total_agents = agents.len();
            let responders = self.max_agents_per_message.min(total_agents);
            let start_index = self.next_agent_index % total_agents;
            self.next_agent_index = (start_index + responders) % total_agents;

            for offset in 0..responders {
                let idx = (start_index + offset) % total_agents;
                let agent = &agents[idx];
                self.app_state.log_debug(
                    "chat_bot",
                    &format!(
                        "🎯 Selected agent [{} of {}]: {} ({})",
                        offset + 1,
                        responders,
                        agent.name,
                        agent.model
                    ),
                );

                if let Err(e) = self.respond_with_agent(agent, msg, &context, &config).await {
                    self.app_state
                        .log_error("chat_bot", &format!("Agent {} error: {}", agent.name, e));
                }
            }
        }

        Ok(())
    }

    async fn respond_with_agent(
        &self,
        agent: &Agent,
        msg: &Message,
        context: &str,
        config: &crate::config::AppConfig,
    ) -> Result<(), String> {
        let base_prompt = prompt::compose_system_prompt(&agent.system_prompt);
        let prompt = if context.is_empty() {
            format!(
                "{}\n\nLatest human message from {}:\n{}\n\nRespond as Pragmatic Sentinel with concise, pragmatic guidance.",
                base_prompt,
                msg.author,
                msg.content
            )
        } else {
            format!(
                "{}\n\n# Recent Conversation\n{}\n\n# Latest human message from {}\n{}\n\nRespond as Pragmatic Sentinel with concise, pragmatic guidance grounded in the above context.",
                base_prompt,
                context,
                msg.author,
                msg.content
            )
        };

        self.app_state.log_network(
            "chat_bot",
            &format!("→ {}:{}", config.ollama_url, agent.model),
        );

        match ollama::ask_ollama_with_auth(
            &config.ollama_url,
            &agent.model,
            prompt,
            Some(("CouncilOfDicks", "")),
        )
        .await
        {
            Ok(response) => {
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

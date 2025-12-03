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
        // Build context
        let messages = self
            .app_state
            .channel_manager
            .get_messages(ChannelType::General, 5, 0)
            .unwrap_or_default();
            
        let context = self.build_context(&messages);

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
        let agents = self.agent_pool.list_agents().await;
        if agents.is_empty() {
            return;
        }

        let active_agents: Vec<Agent> = agents.into_iter().filter(|a| a.active).collect();
        if active_agents.is_empty() {
            return;
        }

        // Pick next agents (round robin)
        let mut selected_agents = Vec::new();
        for _ in 0..self.max_agents_per_message {
            if self.next_agent_index >= active_agents.len() {
                self.next_agent_index = 0;
            }
            selected_agents.push(active_agents[self.next_agent_index].clone());
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
                status.current_reasoning = Some("Analyzing context...".to_string());
            }

            let config = self.app_state.get_config();
            
            // Simulate reasoning steps (for UI effect)
            let reasoning_steps = [
                "Reading message...",
                "Consulting knowledge bank...",
                "Formulating response...",
                "Drafting reply...",
            ];

            for step in reasoning_steps {
                {
                    let mut status = self.app_state.chat_bot_status.lock().unwrap();
                    status.current_reasoning = Some(step.to_string());
                }
                sleep(Duration::from_millis(500)).await;
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
                "{}\n\nLatest human message from {}:\n{}\n\nRespond as {}. Start your response by mentioning the participants you are addressing (e.g. @human_user, @technical_architect). Keep it concise and pragmatic.",
                base_prompt,
                msg.author,
                msg.content,
                agent.name
            )
        } else {
            format!(
                "{}\n\n# Recent Conversation\n{}\n\n# Latest human message from {}\n{}\n\nRespond as {}. Start your response by mentioning the participants you are addressing (e.g. @human_user, @technical_architect). Keep it concise and pragmatic, grounded in the above context.",
                base_prompt,
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

        match ollama::ask_ollama_with_auth(
            &config.ollama_url,
            &agent.model,
            prompt,
            Some(("CouncilOfDicks", "")),
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

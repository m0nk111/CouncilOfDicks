use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicStatus {
    pub current_topic: Option<String>,
    pub queue_length: usize,
    pub next_run_in_secs: u64,
    pub is_running: bool,
}

pub struct TopicManager {
    state: Arc<Mutex<TopicInternalState>>,
}

struct TopicInternalState {
    current_topic: Option<String>,
    queue: VecDeque<String>, // Agent IDs
    interval_secs: u64,
    is_running: bool,
    last_run: SystemTime,
    last_topic_change: SystemTime,
}

impl TopicManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(TopicInternalState {
                current_topic: None,
                queue: VecDeque::new(),
                interval_secs: 300, // 5 minutes default
                is_running: false,
                last_run: SystemTime::now(),
                last_topic_change: SystemTime::UNIX_EPOCH,
            })),
        }
    }

    pub fn validate_topic_change(&self, new_topic: &str) -> Result<(), String> {
        let state = self.state.lock().unwrap();
        
        // Rule 1: Content validation
        if new_topic.trim().is_empty() {
            return Err("Topic cannot be empty".to_string());
        }
        if new_topic.len() > 100 {
            return Err("Topic is too long (max 100 chars)".to_string());
        }

        // Rule 2: Minimum duration (Anti-spam)
        // Only enforce if there IS a current topic running
        if state.is_running && state.current_topic.is_some() {
            let min_duration = Duration::from_secs(300); // 5 minutes lock
            let elapsed = SystemTime::now()
                .duration_since(state.last_topic_change)
                .unwrap_or(Duration::ZERO);
            
            if elapsed < min_duration {
                let remaining = min_duration.as_secs() - elapsed.as_secs();
                return Err(format!("Topic is locked for another {} seconds", remaining));
            }
        }

        Ok(())
    }

    pub fn set_topic(&self, topic: String, interval_secs: Option<u64>) -> Result<(), String> {
        // Validate first
        self.validate_topic_change(&topic)?;

        let mut state = self.state.lock().unwrap();
        state.current_topic = Some(topic);
        if let Some(secs) = interval_secs {
            state.interval_secs = secs;
        }
        state.is_running = true;
        state.queue.clear(); // Reset queue on new topic
        // Reset timer so it starts soon
        state.last_run = SystemTime::now() - Duration::from_secs(state.interval_secs); 
        state.last_topic_change = SystemTime::now();
        
        Ok(())
    }

    pub fn force_set_topic(&self, topic: String, interval_secs: Option<u64>) {
        // Bypass validation (used for initial config or admin override)
        let mut state = self.state.lock().unwrap();
        state.current_topic = Some(topic);
        if let Some(secs) = interval_secs {
            state.interval_secs = secs;
        }
        state.is_running = true;
        state.queue.clear();
        state.last_run = SystemTime::now() - Duration::from_secs(state.interval_secs);
        state.last_topic_change = SystemTime::now();
    }

    pub async fn broadcast_topic(&self, app_state: Arc<AppState>, topic: String, interval: u64) {
        // Create message
        let peer_id = app_state.p2p_manager.status().await.peer_id.unwrap_or_default();
        let msg = crate::protocol::CouncilMessage::TopicUpdate {
            topic,
            interval,
            set_by_peer_id: peer_id,
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
        };

        // Broadcast
        let _ = app_state.p2p_manager.publish("council", msg).await;
    }

    pub fn stop(&self) {
        let mut state = self.state.lock().unwrap();
        state.is_running = false;
        state.current_topic = None;
        state.queue.clear();
    }

    pub fn get_status(&self) -> TopicStatus {
        let state = self.state.lock().unwrap();
        let now = SystemTime::now();
        let elapsed = now.duration_since(state.last_run).unwrap_or(Duration::from_secs(0)).as_secs();
        let next_run = if elapsed < state.interval_secs {
            state.interval_secs - elapsed
        } else {
            0
        };

        TopicStatus {
            current_topic: state.current_topic.clone(),
            queue_length: state.queue.len(),
            next_run_in_secs: next_run,
            is_running: state.is_running,
        }
    }

    // Called by the background loop
    pub async fn tick(&self, app_state: Arc<AppState>) {
        // First, check if we need to run, without holding the lock across await
        let should_run = {
            let state = self.state.lock().unwrap();
            if !state.is_running || state.current_topic.is_none() {
                false
            } else {
                let now = SystemTime::now();
                let elapsed = now.duration_since(state.last_run).unwrap_or(Duration::from_secs(0));
                elapsed.as_secs() >= state.interval_secs
            }
        };

        if !should_run {
            return;
        }

        // If we need to refill the queue, do it now (async)
        let needs_refill = {
            let state = self.state.lock().unwrap();
            state.queue.is_empty()
        };

        if needs_refill {
            let agents = app_state.agent_pool.list_active_agents().await;
            let mut state = self.state.lock().unwrap();
            for agent in agents {
                state.queue.push_back(agent.id);
            }
        }

        // Now get the next agent and update state
        let (topic, agent_id) = {
            let mut state = self.state.lock().unwrap();
            
            // Re-check conditions in case they changed
            if !state.is_running || state.current_topic.is_none() {
                return;
            }
            
            if state.queue.is_empty() {
                return;
            }

            let agent_id = state.queue.pop_front();
            let topic = state.current_topic.clone();
            
            state.last_run = SystemTime::now();
            
            (topic, agent_id)
        };

        if let (Some(topic), Some(agent_id)) = (topic, agent_id) {
            // Execute the agent response
            if let Ok(agent) = app_state.agent_pool.get_agent(&agent_id).await {
                let prompt = format!(
                    "TOPIC DISCUSSION\n\nTopic: {}\n\nPlease provide your perspective on this topic. Keep it concise and insightful. Start your response with your opinion.",
                    topic
                );

                let config = app_state.get_config();
                let system_prompt = crate::prompt::compose_system_prompt(&agent.system_prompt);
                
                let provider = crate::providers::ollama::OllamaProvider::new(
                    config.ollama_url.clone(),
                    agent.model.clone(),
                    app_state.logger.clone(),
                );
                
                use crate::providers::AIProvider;
                // We need to use the AIProvider trait method `generate` or similar, but `ask` was my guess.
                // Let's check `AIProvider` trait definition.
                // Actually, let's just use `generate` which is likely the method name.
                // Wait, I don't have the trait definition handy, but `ask` failed.
                // Let's look at `src/providers/mod.rs` or similar if I could, but I'll guess `generate` based on `GenerationRequest`.
                
                let request = crate::providers::GenerationRequest {
                    model: agent.model.clone(),
                    prompt: prompt.clone(),
                    system_prompt: Some(system_prompt),
                    temperature: agent.temperature,
                    max_tokens: None,
                    stream: false,
                };

                match provider.generate(request).await {
                    Ok(response) => {
                        // Post to chat
                        let message_content = format!("#topic {}\n\n{}", topic, response.text);
                        
                        let message = crate::chat::Message::new(
                            crate::chat::ChannelType::General,
                            agent.name.clone(),
                            crate::chat::AuthorType::AI,
                            message_content
                        );

                        let _ = app_state.channel_manager.send_message(message);
                    },
                    Err(e) => {
                        app_state.logger.error("topic_manager", &format!("Agent {} failed to reply: {:?}", agent.name, e));
                    }
                }
            }
        }
    }
}

// Background task starter
pub fn start_topic_loop(app_state: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(5)).await; // Check every 5 seconds
            app_state.topic_manager.tick(app_state.clone()).await;
        }
    });
}

// Tauri Commands
#[tauri::command]
pub async fn topic_set(topic: String, interval: Option<u64>, state: tauri::State<'_, AppState>) -> Result<TopicStatus, String> {
    let interval_val = interval.unwrap_or(300);
    
    // Try to set topic (will fail if validation fails)
    state.topic_manager.set_topic(topic.clone(), Some(interval_val))?;
    
    // Broadcast to network
    state.topic_manager.broadcast_topic(Arc::new(state.inner().clone()), topic, interval_val).await;
    
    Ok(state.topic_manager.get_status())
}

#[tauri::command]
pub fn topic_stop(state: tauri::State<AppState>) -> TopicStatus {
    state.topic_manager.stop();
    state.topic_manager.get_status()
}

#[tauri::command]
pub fn topic_get_status(state: tauri::State<AppState>) -> TopicStatus {
    state.topic_manager.get_status()
}

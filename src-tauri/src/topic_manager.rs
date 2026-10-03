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
    pub topic_queue_length: usize,
    pub queued_topics: Vec<String>,
    pub next_run_in_secs: u64,
    pub is_running: bool,
    pub next_agent: Option<String>,
}

pub struct TopicManager {
    state: Arc<Mutex<TopicInternalState>>,
}

#[derive(Debug, Clone)]
struct QueuedTopic {
    topic: String,
    interval_secs: u64,
}

struct TopicInternalState {
    current_topic: Option<String>,
    queue: VecDeque<String>, // Agent IDs
    topic_queue: VecDeque<QueuedTopic>, // Pending topics
    agents_initialized: bool,
    interval_secs: u64,
    is_running: bool,
    last_run: SystemTime,
    last_topic_change: SystemTime,
}

impl Default for TopicManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TopicManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(TopicInternalState {
                current_topic: None,
                queue: VecDeque::new(),
                topic_queue: VecDeque::new(),
                agents_initialized: false,
                interval_secs: 300, // 5 minutes default
                is_running: false,
                last_run: SystemTime::now(),
                last_topic_change: SystemTime::UNIX_EPOCH,
            })),
        }
    }

    fn validate_topic_text(topic: &str) -> Result<(), String> {
        if topic.trim().is_empty() {
            return Err("Topic cannot be empty".to_string());
        }
        if topic.len() > 100 {
            return Err("Topic is too long (max 100 chars)".to_string());
        }
        Ok(())
    }

    pub fn validate_topic_change(&self, new_topic: &str) -> Result<(), String> {
        let state = self.state.lock().unwrap();
        
        // Rule 1: Content validation
        Self::validate_topic_text(new_topic)?;

        // Rule 2: Minimum duration (Anti-spam)
        // Only enforce if there IS a current topic running AND it's not the same topic
        if state.is_running && state.current_topic.is_some() {
            // Allow updating the interval if the topic name is the same
            if let Some(current) = &state.current_topic {
                if current == new_topic {
                    return Ok(());
                }
            }

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
        state.topic_queue.clear(); // Manual override clears queued topics
        state.agents_initialized = false;
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
        state.topic_queue.clear();
        state.agents_initialized = false;
        state.last_run = SystemTime::now() - Duration::from_secs(state.interval_secs);
        state.last_topic_change = SystemTime::now();
    }

    /// Enqueue a topic to be discussed in #topic. Does not replace the current topic.
    /// Posts a system message to #topic showing the current topic queue.
    pub fn enqueue_topic(&self, topic: String) -> Result<(), String> {
        Self::validate_topic_text(&topic)?;

        let mut state = self.state.lock().unwrap();
        // Queued topics run with a 5-minute interval between agents by default.
        state.topic_queue.push_back(QueuedTopic {
            topic,
            interval_secs: 300,
        });
        state.is_running = true;
        // Make it eligible to run soon.
        state.last_run = SystemTime::now() - Duration::from_secs(state.interval_secs);
        Ok(())
    }

    /// Enqueue a topic and immediately post a visible queue update message into #topic.
    pub fn enqueue_topic_with_announcement(
        &self,
        app_state: &Arc<AppState>,
        topic: String,
    ) -> Result<(), String> {
        Self::validate_topic_text(&topic)?;

        let queued_topics = {
            let mut state = self.state.lock().unwrap();
            // Queued topics run with a 5-minute interval between agents by default.
            state.topic_queue.push_back(QueuedTopic {
                topic,
                interval_secs: 300,
            });
            state.is_running = true;
            // Make it eligible to run soon.
            state.last_run = SystemTime::now() - Duration::from_secs(state.interval_secs);
            state
                .topic_queue
                .iter()
                .map(|t| t.topic.clone())
                .collect::<Vec<String>>()
        };

        Self::post_topic_queue_message(app_state, &queued_topics);
        Ok(())
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
        state.topic_queue.clear();
        state.agents_initialized = false;
    }

    pub fn get_status(&self) -> TopicStatus {
        let state = self.state.lock().unwrap();
        let now = SystemTime::now();
        let elapsed = now.duration_since(state.last_run).unwrap_or(Duration::from_secs(0)).as_secs();
        let next_run = state.interval_secs.saturating_sub(elapsed);

        TopicStatus {
            current_topic: state.current_topic.clone(),
            queue_length: state.queue.len(),
            topic_queue_length: state.topic_queue.len(),
            queued_topics: state.topic_queue.iter().map(|t| t.topic.clone()).collect(),
            next_run_in_secs: next_run,
            is_running: state.is_running,
            next_agent: state.queue.front().cloned(),
        }
    }

    fn post_topic_queue_message(app_state: &Arc<AppState>, queued_topics: &[String]) {
        let mut content = String::from("📥 Topic queue updated\n\n");
        if queued_topics.is_empty() {
            content.push_str("(queue is empty)");
        } else {
            for (i, t) in queued_topics.iter().enumerate() {
                content.push_str(&format!("{}. {}\n", i + 1, t));
            }
        }

        let message = crate::chat::Message::new(
            crate::chat::ChannelType::Topic,
            "System".to_string(),
            crate::chat::AuthorType::System,
            content,
        );
        let _ = app_state.channel_manager.send_message(message);
    }

    fn post_agent_queue_message(app_state: &Arc<AppState>, topic: &str, agent_names: &[String]) {
        let mut content = format!("🧾 Agent queue for topic: {}\n\n", topic);
        if agent_names.is_empty() {
            content.push_str("(no active agents)");
        } else {
            for (i, name) in agent_names.iter().enumerate() {
                content.push_str(&format!("{}. {}\n", i + 1, name));
            }
        }
        let message = crate::chat::Message::new(
            crate::chat::ChannelType::Topic,
            "System".to_string(),
            crate::chat::AuthorType::System,
            content,
        );
        let _ = app_state.channel_manager.send_message(message);
    }

    // Called by the background loop
    pub async fn tick(&self, app_state: Arc<AppState>) {
        // Check PoHV status first - Safety Mechanism
        if app_state.pohv_system.is_locked() {
            // If locked, we do not process any topics.
            // We could also log a warning if we haven't recently.
            return;
        }

        // If no current topic but queued topics exist, promote next topic.
        {
            let mut state = self.state.lock().unwrap();
            if state.current_topic.is_none() && !state.topic_queue.is_empty() {
                let next = state.topic_queue.pop_front();
                if let Some(queued) = next {
                    state.current_topic = Some(queued.topic);
                    state.queue.clear();
                    state.agents_initialized = false;
                    state.is_running = true;
                    state.interval_secs = queued.interval_secs;
                    state.last_run = SystemTime::now() - Duration::from_secs(state.interval_secs);

                    let remaining: Vec<String> =
                        state.topic_queue.iter().map(|t| t.topic.clone()).collect();
                    drop(state);
                    Self::post_topic_queue_message(&app_state, &remaining);
                }
            }
        }

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

        // If we need to initialize the agent queue for this topic, do it once.
        let needs_init = {
            let state = self.state.lock().unwrap();
            state.current_topic.is_some() && !state.agents_initialized
        };

        if needs_init {
            let agents = app_state.agent_pool.list_active_agents().await;
            let (topic, agent_names) = {
                let mut state = self.state.lock().unwrap();
                if state.current_topic.is_none() {
                    return;
                }
                state.queue.clear();
                let mut names = Vec::new();
                for agent in agents {
                    state.queue.push_back(agent.id);
                    names.push(agent.name);
                }
                state.agents_initialized = true;
                (state.current_topic.clone().unwrap_or_default(), names)
            };

            Self::post_agent_queue_message(&app_state, &topic, &agent_names);
        }

        // Now get the next agent and update state
        let (topic, agent_id, finished_topic) = {
            let mut state = self.state.lock().unwrap();

            // Re-check conditions in case they changed
            if !state.is_running || state.current_topic.is_none() {
                return;
            }

            // If queue is empty and we already initialized for this topic, the round is finished.
            if state.queue.is_empty() {
                if state.agents_initialized {
                    let finished = state.current_topic.clone();
                    state.current_topic = None;
                    state.agents_initialized = false;

                    // If no queued topics remain, stop running.
                    if state.topic_queue.is_empty() {
                        state.is_running = false;
                    }

                    state.last_run = SystemTime::now();
                    (None, None, finished)
                } else {
                    return;
                }
            } else {
                let agent_id = state.queue.pop_front();
                let topic = state.current_topic.clone();
                state.last_run = SystemTime::now();
                (topic, agent_id, None)
            }
        };

        if let Some(done) = finished_topic {
            let message = crate::chat::Message::new(
                crate::chat::ChannelType::Topic,
                "System".to_string(),
                crate::chat::AuthorType::System,
                format!("✅ Topic round complete: {}", done),
            );
            let _ = app_state.channel_manager.send_message(message);
            return;
        }

        if let (Some(topic), Some(agent_id)) = (topic, agent_id) {
            // Execute the agent response
            if let Ok(agent) = app_state.agent_pool.get_agent(&agent_id).await {
                // Build RAG context if available
                let mut context_str = String::new();
                
                // 1. Get recent discussion context from the topic channel
                if let Ok(messages) = app_state.channel_manager.get_messages(crate::chat::ChannelType::Topic, 10, 0) {
                    if !messages.is_empty() {
                        context_str.push_str("\n\nRECENT DISCUSSION:\n");
                        for msg in messages.iter().rev() { // Reverse to chronological order
                            context_str.push_str(&format!("{}: {}\n", msg.author, msg.content));
                        }
                    }
                }

                // 2. Get Knowledge Bank context if available
                if let Some(kb) = &app_state.knowledge_bank {
                    if let Ok(rag) = kb.build_rag_context(&topic, 3).await {
                        if !rag.relevant_decisions.is_empty() {
                            context_str.push_str(&format!("\n\nRELEVANT PAST DECISIONS:\n{}", rag.context_text));
                        }
                    }
                }

                let prompt = format!(
                    "TOPIC DISCUSSION\n\nTopic: {}\n{}\n\nPlease provide your perspective on this topic. Keep it concise and insightful. Start your response with your opinion. If relevant, reference the past decisions provided. Respond to previous points if applicable.",
                    topic, context_str
                );

                let config = app_state.get_config();
                // Use topic-specific system prompt WITHOUT TCOD framing
                let system_prompt = crate::prompt::compose_topic_system_prompt(&agent.system_prompt);

                match crate::provider_dispatch::generate_with_timeout(
                    &agent.provider,
                    &agent.model,
                    prompt.clone(),
                    Some(system_prompt),
                    &config,
                    Some(app_state.logger.clone()),
                    agent.timeout_secs,
                ).await {
                    Ok(response) => {
                        // Post to chat
                        let message_content = format!("#topic {}\n\n{}", topic, response);
                        
                        let message = crate::chat::Message::new(
                            crate::chat::ChannelType::Topic,
                            agent.name.clone(),
                            crate::chat::AuthorType::AI,
                            message_content
                        );

                        let _ = app_state.channel_manager.send_message(message);
                    },
                    Err(e) => {
                        app_state.logger.error("topic_manager", &format!("Agent {} failed to reply: {}", agent.name, e));
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
            sleep(Duration::from_secs(1)).await; // Check every 1 second (queue-driven)
            app_state.topic_manager.tick(app_state.clone()).await;
        }
    });
}

// Tauri Commands
#[tauri::command]
pub async fn topic_set(topic: String, interval: Option<u64>, state: tauri::State<'_, AppState>) -> Result<TopicStatus, String> {
    let interval_val = interval.unwrap_or(600); // Default 10 minutes
    
    // Try to set topic (will fail if validation fails)
    state.topic_manager.set_topic(topic.clone(), Some(interval_val))?;
    
    // Save to Knowledge Bank if available
    if let Some(kb) = &state.knowledge_bank {
        let peer_id = state.p2p_manager.status().await.peer_id.unwrap_or_else(|| "local".to_string());
        if let Err(e) = kb.add_topic(&topic, Some(&peer_id)).await {
            state.logger.warn("topic_manager", &format!("Failed to save topic to history: {}", e));
        }
    }

    // Broadcast to network
    // We need to clone the Arc<AppState> properly. state.inner() returns &AppState.
    // But broadcast_topic takes Arc<AppState>.
    // We can't easily get Arc<AppState> from tauri::State<AppState> directly if it wasn't created as Arc.
    // However, AppState fields are mostly Arcs, so we can just pass the fields we need, or change broadcast_topic signature.
    // But wait, broadcast_topic takes Arc<AppState>.
    // Let's look at how it's called.
    // state.topic_manager.broadcast_topic(Arc::new(state.inner().clone()), topic, interval_val).await;
    // AppState derives Clone? Let's check state.rs.
    // If AppState derives Clone, then state.inner().clone() creates a new AppState struct with cloned Arcs.
    // Then Arc::new(...) wraps it. This is fine.
    
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

#[tauri::command]
pub async fn topic_history(limit: Option<i64>, state: tauri::State<'_, AppState>) -> Result<Vec<(String, i64)>, String> {
    if let Some(kb) = &state.knowledge_bank {
        kb.get_recent_topics(limit.unwrap_or(10)).await
    } else {
        Ok(Vec::new())
    }
}

// Agent Pool Management - Add/remove AI models to chat sessions

use crate::prompt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Statistics tracked per agent
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentStats {
    /// Total number of requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Total input tokens (prompt)
    pub total_input_tokens: u64,
    /// Total output tokens (completion)
    pub total_output_tokens: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Last response time in milliseconds
    pub last_response_time_ms: f64,
    /// Last context size (chars)
    pub last_context_size: usize,
    /// Timestamp of last activity (Unix epoch seconds)
    pub last_activity: u64,
}

impl AgentStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful request
    pub fn record_success(&mut self, input_tokens: u64, output_tokens: u64, response_time_ms: f64, context_size: usize) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.total_input_tokens += input_tokens;
        self.total_output_tokens += output_tokens;
        self.last_response_time_ms = response_time_ms;
        self.last_context_size = context_size;
        self.last_activity = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Update rolling average
        let total = self.successful_requests as f64;
        self.avg_response_time_ms = ((self.avg_response_time_ms * (total - 1.0)) + response_time_ms) / total;
    }

    /// Record a failed request
    pub fn record_failure(&mut self, response_time_ms: f64) {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.last_response_time_ms = response_time_ms;
        self.last_activity = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 100.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Get total tokens used
    pub fn total_tokens(&self) -> u64 {
        self.total_input_tokens + self.total_output_tokens
    }
}

/// Represents a single AI agent that can participate in council sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique identifier
    pub id: String,

    /// Display name (e.g., "The Pragmatist", "Code Reviewer")
    pub name: String,

    /// Unique handle for mentions (e.g., "pragmatic_sentinel")
    #[serde(default)]
    pub handle: String,

    /// Provider type: "ollama", "openai", "openrouter", "google"
    #[serde(default = "default_provider")]
    pub provider: String,

    /// Model to use (e.g., "qwen2.5-coder:7b", "gpt-4o", "gemini-1.5-flash")
    pub model: String,

    /// System prompt explaining context, role, and rules
    pub system_prompt: String,

    /// Tools this agent can use (e.g., ["send_message", "vote", "search_knowledge"])
    pub enabled_tools: Vec<String>,

    /// Temperature for generation (0.0-2.0)
    pub temperature: f32,

    /// Is this agent currently active?
    pub active: bool,

    /// Metadata for UI/sorting
    pub metadata: HashMap<String, String>,
}

fn default_provider() -> String {
    "ollama".to_string()
}

impl Agent {
    /// Create new agent with defaults
    pub fn new(name: String, model: String, system_prompt: String) -> Self {
        let handle = name.to_lowercase().replace(" ", "_");
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            handle,
            provider: "ollama".to_string(),
            model,
            system_prompt,
            enabled_tools: vec!["send_message".to_string(), "vote".to_string()],
            temperature: 0.7,
            active: true,
            metadata: HashMap::new(),
        }
    }

    /// Create agent with specific provider
    pub fn with_provider(name: String, provider: String, model: String, system_prompt: String) -> Self {
        let handle = name.to_lowercase().replace(" ", "_");
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            handle,
            provider,
            model,
            system_prompt,
            enabled_tools: vec!["send_message".to_string(), "vote".to_string()],
            temperature: 0.7,
            active: true,
            metadata: HashMap::new(),
        }
    }

    /// Build full prompt with system context + user question
    pub fn build_prompt(&self, user_message: &str, context: Option<&str>) -> String {
        let mut prompt = prompt::compose_system_prompt(&self.system_prompt);

        if let Some(ctx) = context {
            prompt.push_str("\n\n# Previous Discussion:\n");
            prompt.push_str(ctx);
        }

        prompt.push_str("\n\n# Current Question:\n");
        prompt.push_str(user_message);

        prompt
    }
}

/// Manages a pool of agents for council sessions
#[derive(Clone)]
pub struct AgentPool {
    agents: Arc<Mutex<HashMap<String, Agent>>>,
    stats: Arc<Mutex<HashMap<String, AgentStats>>>,
}

impl AgentPool {
    /// Create new empty agent pool
    pub fn new() -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add agent to pool
    pub async fn add_agent(&self, agent: Agent) -> Result<String, String> {
        let mut agents = self.agents.lock().await;
        let agent_id = agent.id.clone();

        // Check for duplicate names
        if agents
            .values()
            .any(|a| a.name == agent.name && a.id != agent.id)
        {
            return Err(format!("Agent with name '{}' already exists", agent.name));
        }

        agents.insert(agent_id.clone(), agent);
        
        // Initialize stats for new agent
        let mut stats = self.stats.lock().await;
        stats.entry(agent_id.clone()).or_insert_with(AgentStats::new);
        Ok(agent_id)
    }

    /// Remove agent from pool
    pub async fn remove_agent(&self, agent_id: &str) -> Result<(), String> {
        let mut agents = self.agents.lock().await;
        agents
            .remove(agent_id)
            .ok_or_else(|| format!("Agent not found: {}", agent_id))?;
        Ok(())
    }

    /// Get agent by ID
    pub async fn get_agent(&self, agent_id: &str) -> Result<Agent, String> {
        let agents = self.agents.lock().await;
        agents
            .get(agent_id)
            .cloned()
            .ok_or_else(|| format!("Agent not found: {}", agent_id))
    }

    /// Update existing agent
    pub async fn update_agent(&self, agent: Agent) -> Result<(), String> {
        let mut agents = self.agents.lock().await;
        if !agents.contains_key(&agent.id) {
            return Err(format!("Agent not found: {}", agent.id));
        }
        agents.insert(agent.id.clone(), agent);
        Ok(())
    }

    /// List all agents
    pub async fn list_agents(&self) -> Vec<Agent> {
        let agents = self.agents.lock().await;
        agents.values().cloned().collect()
    }

    /// List only active agents
    pub async fn list_active_agents(&self) -> Vec<Agent> {
        let agents = self.agents.lock().await;
        agents.values().filter(|a| a.active).cloned().collect()
    }

    /// Get agents by IDs
    pub async fn get_agents_by_ids(&self, agent_ids: &[String]) -> Result<Vec<Agent>, String> {
        let agents = self.agents.lock().await;
        let mut result = Vec::new();

        for id in agent_ids {
            let agent = agents
                .get(id)
                .ok_or_else(|| format!("Agent not found: {}", id))?;
            result.push(agent.clone());
        }

        Ok(result)
    }

    /// Get stats for a specific agent
    pub async fn get_agent_stats(&self, agent_id: &str) -> AgentStats {
        let stats = self.stats.lock().await;
        stats.get(agent_id).cloned().unwrap_or_default()
    }

    /// Get stats for all agents
    pub async fn get_all_stats(&self) -> HashMap<String, AgentStats> {
        let stats = self.stats.lock().await;
        stats.clone()
    }

    /// Record a successful request for an agent
    pub async fn record_success(&self, agent_id: &str, input_tokens: u64, output_tokens: u64, response_time_ms: f64, context_size: usize) {
        let mut stats = self.stats.lock().await;
        let agent_stats = stats.entry(agent_id.to_string()).or_insert_with(AgentStats::new);
        agent_stats.record_success(input_tokens, output_tokens, response_time_ms, context_size);
    }

    /// Record a failed request for an agent
    pub async fn record_failure(&self, agent_id: &str, response_time_ms: f64) {
        let mut stats = self.stats.lock().await;
        let agent_stats = stats.entry(agent_id.to_string()).or_insert_with(AgentStats::new);
        agent_stats.record_failure(response_time_ms);
    }

    /// Start timing a request (returns instant for later measurement)
    pub fn start_request(&self) -> Instant {
        Instant::now()
    }
}

impl Default for AgentPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool that agents can use to interact with the council
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool identifier (e.g., "send_message", "vote", "search_knowledge")
    pub name: String,

    /// Human-readable description for the AI
    pub description: String,

    /// JSON schema for parameters
    pub parameters: serde_json::Value,
}

impl Tool {
    /// Create standard "send_message" tool
    pub fn send_message() -> Self {
        Self {
            name: "send_message".to_string(),
            description: "Send a message to the council chat. Use this to share your thoughts, respond to others, or contribute to the discussion.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message content to send"
                    }
                },
                "required": ["message"]
            }),
        }
    }

    /// Create standard "vote" tool
    pub fn vote() -> Self {
        Self {
            name: "vote".to_string(),
            description:
                "Cast a vote on the current question. Use this when you've reached a conclusion."
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "vote": {
                        "type": "string",
                        "description": "Your vote (e.g., 'yes', 'no', 'abstain', or a custom answer)"
                    },
                    "reasoning": {
                        "type": "string",
                        "description": "Brief explanation of your vote"
                    }
                },
                "required": ["vote", "reasoning"]
            }),
        }
    }

    /// Create standard "search_knowledge" tool
    pub fn search_knowledge() -> Self {
        Self {
            name: "search_knowledge".to_string(),
            description:
                "Search the council's knowledge bank for relevant past decisions and context."
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum results to return",
                        "default": 5
                    }
                },
                "required": ["query"]
            }),
        }
    }

    /// Get all standard tools
    pub fn standard_tools() -> Vec<Tool> {
        vec![Self::send_message(), Self::vote(), Self::search_knowledge()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_creation() {
        let agent = Agent::new(
            "Test Agent".to_string(),
            "qwen2.5-coder:7b".to_string(),
            "You are a helpful AI assistant.".to_string(),
        );

        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.model, "qwen2.5-coder:7b");
        assert!(agent.active);
        assert_eq!(agent.temperature, 0.7);
    }

    #[tokio::test]
    async fn test_agent_pool_add_remove() {
        let pool = AgentPool::new();

        let agent = Agent::new(
            "Test Agent".to_string(),
            "qwen2.5-coder:7b".to_string(),
            "System prompt".to_string(),
        );

        let agent_id = pool.add_agent(agent.clone()).await.unwrap();

        // Check agent exists
        let retrieved = pool.get_agent(&agent_id).await.unwrap();
        assert_eq!(retrieved.name, "Test Agent");

        // List agents
        let agents = pool.list_agents().await;
        assert_eq!(agents.len(), 1);

        // Remove agent
        pool.remove_agent(&agent_id).await.unwrap();
        let agents = pool.list_agents().await;
        assert_eq!(agents.len(), 0);
    }

    #[tokio::test]
    async fn test_duplicate_agent_names() {
        let pool = AgentPool::new();

        let agent1 = Agent::new(
            "Duplicate".to_string(),
            "model1".to_string(),
            "prompt1".to_string(),
        );

        let agent2 = Agent::new(
            "Duplicate".to_string(),
            "model2".to_string(),
            "prompt2".to_string(),
        );

        pool.add_agent(agent1).await.unwrap();
        let result = pool.add_agent(agent2).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[tokio::test]
    async fn test_agent_prompt_building() {
        let agent = Agent::new(
            "Test".to_string(),
            "model".to_string(),
            "You are an expert in testing.".to_string(),
        );

        let prompt = agent.build_prompt("What is TDD?", None);
        assert!(prompt.contains("You are an expert in testing"));
        assert!(prompt.contains("What is TDD?"));

        let prompt_with_context = agent.build_prompt(
            "Continue the discussion",
            Some("Previous: We discussed unit tests"),
        );
        assert!(prompt_with_context.contains("Previous Discussion"));
        assert!(prompt_with_context.contains("unit tests"));
    }

    #[tokio::test]
    async fn test_standard_tools() {
        let tools = Tool::standard_tools();
        assert_eq!(tools.len(), 3);

        let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"send_message".to_string()));
        assert!(tool_names.contains(&"vote".to_string()));
        assert!(tool_names.contains(&"search_knowledge".to_string()));
    }
}

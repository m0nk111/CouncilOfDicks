// Unit tests for Council Of Dicks backend

#[cfg(test)]
mod tests {
    use crate::config::AppConfig;
    use crate::logger::{LogLevel, Logger};
    use crate::metrics::MetricsCollector;
    use crate::state::AppState;

    #[test]
    fn test_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.ollama_url, "http://192.168.1.5:11434");
        assert_eq!(config.ollama_model, "qwen3-coder-30b-q4km-32k:latest");
        assert_eq!(config.debug_enabled, true);
    }

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();

        let retrieved_config = state.get_config();
        assert!(!retrieved_config.ollama_url.is_empty());
        assert!(!retrieved_config.ollama_model.is_empty());
    }

    #[test]
    fn test_debug_toggle() {
        let state = AppState::new();
        let config = state.get_config();
        // Check current debug state (default is true from AppConfig::default())
        assert!(config.debug_enabled);
    }

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new(true);
        // Logger created successfully
        logger.debug("test", "Debug logging works");
        logger.info("test", "Info logging works");
    }

    #[test]
    fn test_logger_debug_toggle() {
        let logger = Logger::new(false);
        logger.set_debug_enabled(true);
        // Should not panic
        logger.debug("test", "This should log now");
    }

    #[tokio::test]
    async fn test_knowledge_bank_persistence() {
        use crate::knowledge::KnowledgeBank;
        use crate::protocol::{CouncilSession, SessionStatus};
        use crate::reputation::{AgentReputation, AgentTier, ReputationScore};
        use std::sync::Arc;

        // 1. Initialize in-memory DB
        let logger = Arc::new(Logger::new(true));
        let kb = KnowledgeBank::new(
            "sqlite::memory:",
            logger,
            "http://localhost:11434".to_string(),
        )
        .await
        .expect("Failed to create KnowledgeBank");

        // 2. Test Session Persistence
        let session = CouncilSession {
            id: "test-session-1".to_string(),
            question: "What is the meaning of life?".to_string(),
            responses: vec![],
            commitments: vec![],
            reveals: vec![],
            consensus: Some("42".to_string()),
            status: SessionStatus::ConsensusReached,
            created_at: 1234567890,
        };

        kb.save_session(&session).await.expect("Failed to save session");

        let loaded_sessions = kb.load_sessions().await.expect("Failed to load sessions");
        assert_eq!(loaded_sessions.len(), 1);
        assert_eq!(loaded_sessions[0].id, "test-session-1");
        assert_eq!(loaded_sessions[0].consensus, Some("42".to_string()));

                // 3. Test Reputation Persistence
        let reputation = AgentReputation {
            agent_id: "agent-007".to_string(),
            score: ReputationScore {
                accuracy: 0.9,
                reasoning: 0.8,
                contribution: 0.7,
                total_votes: 10,
                successful_consensus: 9,
            },
            tier: AgentTier::Prime,
            last_updated: 1234567890,
        };

        kb.save_reputation(&reputation).await.expect("Failed to save reputation");

        let loaded_reputations = kb.load_reputations().await.expect("Failed to load reputations");
        assert_eq!(loaded_reputations.len(), 1);
        assert_eq!(loaded_reputations[0].agent_id, "agent-007");
        assert_eq!(loaded_reputations[0].tier, AgentTier::Prime);
    }

    #[test]
    fn test_metrics_initialization() {
        let metrics = MetricsCollector::new();
        let stats = metrics.get_metrics();

        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
        assert_eq!(stats.average_response_time_ms, 0.0);
    }

    #[test]
    fn test_metrics_success_tracking() {
        let mut metrics = MetricsCollector::new();

        let start = metrics.start_request();
        std::thread::sleep(std::time::Duration::from_millis(10));
        metrics.record_success(start);

        let stats = metrics.get_metrics();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.failed_requests, 0);
        assert!(stats.average_response_time_ms > 0.0);
    }

    #[test]
    fn test_metrics_failure_tracking() {
        let mut metrics = MetricsCollector::new();

        let start = metrics.start_request();
        metrics.record_failure(start);

        let stats = metrics.get_metrics();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 1);
    }

    #[test]
    fn test_metrics_multiple_requests() {
        let mut metrics = MetricsCollector::new();

        // Record 3 successful requests
        for _ in 0..3 {
            let start = metrics.start_request();
            std::thread::sleep(std::time::Duration::from_millis(5));
            metrics.record_success(start);
        }

        // Record 1 failed request
        let start = metrics.start_request();
        metrics.record_failure(start);

        let stats = metrics.get_metrics();
        assert_eq!(stats.total_requests, 4);
        assert_eq!(stats.successful_requests, 3);
        assert_eq!(stats.failed_requests, 1);
    }

    #[test]
    fn test_log_level_emoji() {
        assert_eq!(LogLevel::Debug.emoji(), "🐛");
        assert_eq!(LogLevel::Info.emoji(), "ℹ️");
        assert_eq!(LogLevel::Error.emoji(), "❌");
        assert_eq!(LogLevel::Success.emoji(), "✅");
        assert_eq!(LogLevel::Network.emoji(), "📡");
    }
}

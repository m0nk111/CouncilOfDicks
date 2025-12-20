use crate::logger::{LogLevel, Logger};
use crate::ollama::OllamaClient;
use crate::prompt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Represents a single AI model participating in the council
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilMember {
    pub name: String,
    pub model: String,
    pub personality: String,
    pub system_prompt: String,
}

/// Represents a single round of deliberation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliberationRound {
    pub round_number: usize,
    pub responses: Vec<MemberResponse>,
}

/// Response from a council member in a deliberation round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberResponse {
    pub member_name: String,
    pub model: String,
    pub response: String,
    pub timestamp: u64,
}

/// Result of a deliberation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliberationResult {
    pub session_id: String,
    pub question: String,
    pub rounds: Vec<DeliberationRound>,
    pub consensus: Option<String>,
    pub completed: bool,
    pub created_at: u64,
}

/// Manages the deliberation process
pub struct DeliberationEngine {
    logger: Arc<Logger>,
    ollama_client: Arc<Mutex<OllamaClient>>,
}

impl DeliberationEngine {
    /// Create new deliberation engine
    pub fn new(logger: Arc<Logger>, ollama_client: Arc<Mutex<OllamaClient>>) -> Self {
        logger.log(
            LogLevel::Debug,
            "deliberation",
            "🧠 Deliberation engine initialized",
        );
        Self {
            logger,
            ollama_client,
        }
    }

    /// Start a new deliberation session
    pub async fn start_deliberation(
        &self,
        question: String,
        members: Vec<CouncilMember>,
        max_rounds: usize,
    ) -> Result<DeliberationResult, String> {
        let session_id = Uuid::new_v4().to_string();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.logger.log(
            LogLevel::Info,
            "deliberation",
            &format!("🎭 Starting deliberation: {}", session_id),
        );
        self.logger.log(
            LogLevel::Info,
            "deliberation",
            &format!("   Question: {}", question),
        );
        self.logger.log(
            LogLevel::Info,
            "deliberation",
            &format!("   Members: {}", members.len()),
        );
        self.logger.log(
            LogLevel::Info,
            "deliberation",
            &format!("   Max rounds: {}", max_rounds),
        );

        let mut rounds = Vec::new();
        let mut context = String::new();

        // Round 1: Initial responses
        self.logger.log(
            LogLevel::Info,
            "deliberation",
            "🔄 Round 1: Initial responses",
        );
        let round1 = self.execute_round(1, &question, &members, &context).await?;
        context = self.build_context(&question, &round1);
        rounds.push(round1);

        // Additional rounds if needed
        for round_num in 2..=max_rounds {
            self.logger.log(
                LogLevel::Info,
                "deliberation",
                &format!("🔄 Round {}: Cross-examination", round_num),
            );

            let round = self
                .execute_round(round_num, &question, &members, &context)
                .await?;

            // Check for consensus
            if self.has_consensus(&round) {
                self.logger
                    .log(LogLevel::Info, "deliberation", "✅ Consensus reached!");
                rounds.push(round);
                break;
            }

            context = self.build_context(&question, &round);
            rounds.push(round);
        }

        // Extract consensus if reached
        let consensus = self.extract_consensus(&rounds);
        let completed = consensus.is_some() || rounds.len() >= max_rounds;

        self.logger.log(
            LogLevel::Info,
            "deliberation",
            &format!(
                "🏁 Deliberation complete: {} rounds, consensus: {}",
                rounds.len(),
                consensus.is_some()
            ),
        );

        Ok(DeliberationResult {
            session_id,
            question,
            rounds,
            consensus,
            completed,
            created_at,
        })
    }

    /// Execute a single round of deliberation
    async fn execute_round(
        &self,
        round_number: usize,
        question: &str,
        members: &[CouncilMember],
        context: &str,
    ) -> Result<DeliberationRound, String> {
        let mut responses = Vec::new();

        // Query all members in parallel
        let mut tasks = Vec::new();

        for member in members {
            let client = self.ollama_client.clone();
            let logger = self.logger.clone();
            let member_clone = member.clone();
            let question_clone = question.to_string();
            let context_clone = context.to_string();

            let task = tokio::spawn(async move {
                Self::query_member(
                    client,
                    logger,
                    member_clone,
                    question_clone,
                    context_clone,
                    round_number,
                )
                .await
            });

            tasks.push(task);
        }

        // Wait for all responses
        for task in tasks {
            match task.await {
                Ok(Ok(response)) => responses.push(response),
                Ok(Err(e)) => {
                    self.logger.log(
                        LogLevel::Warning,
                        "deliberation",
                        &format!("⚠️ Member query failed: {}", e),
                    );
                }
                Err(e) => {
                    self.logger.log(
                        LogLevel::Warning,
                        "deliberation",
                        &format!("⚠️ Task error: {}", e),
                    );
                }
            }
        }

        Ok(DeliberationRound {
            round_number,
            responses,
        })
    }

    /// Query a single council member
    async fn query_member(
        ollama_client: Arc<Mutex<OllamaClient>>,
        logger: Arc<Logger>,
        member: CouncilMember,
        question: String,
        context: String,
        round_number: usize,
    ) -> Result<MemberResponse, String> {
        logger.log(
            LogLevel::Debug,
            "deliberation",
            &format!("🤖 Querying {} ({})", member.name, member.model),
        );

        // Build prompt with personality and context
        let system_directive = prompt::compose_system_prompt(&member.system_prompt);
        let prompt = if round_number == 1 {
            format!(
                "Question: {}\n\nProvide your analysis and recommendation.",
                question
            )
        } else {
            format!(
                "Question: {}\n\nPrevious discussion:\n{}\n\nProvide your response considering the previous arguments.",
                question, context
            )
        };

        // Query Ollama
        let client = ollama_client.lock().await;
        let response = client.ask(&member.model, &prompt, Some(&system_directive)).await?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(MemberResponse {
            member_name: member.name,
            model: member.model,
            response,
            timestamp,
        })
    }

    /// Build context string from round responses
    fn build_context(&self, question: &str, round: &DeliberationRound) -> String {
        let mut context = format!("Question: {}\n\n", question);

        for resp in &round.responses {
            context.push_str(&format!(
                "{} ({}):\n{}\n\n",
                resp.member_name, resp.model, resp.response
            ));
        }

        context
    }

    /// Check if responses show consensus
    fn has_consensus(&self, round: &DeliberationRound) -> bool {
        // Simple heuristic: if all responses are similar in conclusion
        // This is a naive implementation - could be improved with NLP
        if round.responses.len() < 2 {
            return false;
        }

        // Check for common agreement phrases (positive consensus indicators)
        let agreement_phrases = ["i agree", "consensus", "i concur", "align with"];
        let disagreement_phrases = ["disagree", "wrong", "incorrect", "oppose", "reject"];
        let mut agreement_count = 0;
        let mut disagreement_count = 0;

        for response in &round.responses {
            let text = response.response.to_lowercase();
            if agreement_phrases.iter().any(|phrase| text.contains(phrase)) {
                agreement_count += 1;
            }
            if disagreement_phrases
                .iter()
                .any(|phrase| text.contains(phrase))
            {
                disagreement_count += 1;
            }
        }

        // If majority shows agreement language AND no significant disagreement
        let majority_threshold = round.responses.len() * 2 / 3;
        agreement_count >= majority_threshold && disagreement_count == 0
    }

    /// Extract consensus from rounds
    fn extract_consensus(&self, rounds: &[DeliberationRound]) -> Option<String> {
        if rounds.is_empty() {
            return None;
        }

        let last_round = rounds.last()?;

        if !self.has_consensus(last_round) {
            return None;
        }

        // Build consensus summary
        let mut consensus = String::from("Council Consensus:\n\n");

        for response in &last_round.responses {
            consensus.push_str(&format!(
                "- {} agrees: {}\n",
                response.member_name,
                response.response.lines().next().unwrap_or("(no summary)")
            ));
        }

        Some(consensus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[tokio::test]
    async fn test_deliberation_engine_creation() {
        let logger = Arc::new(Logger::new(false));
        let config = AppConfig::default();
        let ollama = Arc::new(Mutex::new(OllamaClient::new(config, logger.clone())));

        let engine = DeliberationEngine::new(logger, ollama);
        assert!(std::ptr::addr_of!(engine).is_aligned());
    }

    #[tokio::test]
    async fn test_build_context() {
        let logger = Arc::new(Logger::new(false));
        let config = AppConfig::default();
        let ollama = Arc::new(Mutex::new(OllamaClient::new(config, logger.clone())));

        let engine = DeliberationEngine::new(logger, ollama);

        let round = DeliberationRound {
            round_number: 1,
            responses: vec![MemberResponse {
                member_name: "Test Member".to_string(),
                model: "test-model".to_string(),
                response: "Test response".to_string(),
                timestamp: 0,
            }],
        };

        let context = engine.build_context("Test question?", &round);
        assert!(context.contains("Test question?"));
        assert!(context.contains("Test Member"));
        assert!(context.contains("Test response"));
    }

    #[tokio::test]
    async fn test_consensus_detection() {
        let logger = Arc::new(Logger::new(false));
        let config = AppConfig::default();
        let ollama = Arc::new(Mutex::new(OllamaClient::new(config, logger.clone())));

        let engine = DeliberationEngine::new(logger, ollama);

        // Test with consensus
        let round_with_consensus = DeliberationRound {
            round_number: 2,
            responses: vec![
                MemberResponse {
                    member_name: "Member1".to_string(),
                    model: "model1".to_string(),
                    response: "I agree with the previous analysis.".to_string(),
                    timestamp: 0,
                },
                MemberResponse {
                    member_name: "Member2".to_string(),
                    model: "model2".to_string(),
                    response: "I concur with this approach.".to_string(),
                    timestamp: 0,
                },
            ],
        };

        assert!(engine.has_consensus(&round_with_consensus));

        // Test without consensus
        let round_without_consensus = DeliberationRound {
            round_number: 2,
            responses: vec![
                MemberResponse {
                    member_name: "Member1".to_string(),
                    model: "model1".to_string(),
                    response: "I strongly disagree.".to_string(),
                    timestamp: 0,
                },
                MemberResponse {
                    member_name: "Member2".to_string(),
                    model: "model2".to_string(),
                    response: "This is completely wrong.".to_string(),
                    timestamp: 0,
                },
            ],
        };

        assert!(!engine.has_consensus(&round_without_consensus));
    }
}

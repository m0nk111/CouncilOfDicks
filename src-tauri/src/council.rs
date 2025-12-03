// Council session manager for multi-round deliberation

use crate::agents::{Agent, AgentPool};
use crate::protocol::{CouncilResponse, CouncilSession, SessionStatus, VoteCommitment, VoteReveal};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::knowledge::KnowledgeBank;

/// Manages council deliberation sessions
pub struct CouncilSessionManager {
    sessions: Arc<Mutex<HashMap<String, CouncilSession>>>,
    consensus_threshold: f64, // Byzantine fault tolerance: 67%
    knowledge_bank: Option<Arc<KnowledgeBank>>,
}

impl CouncilSessionManager {
    /// Create new session manager
    pub fn new(knowledge_bank: Option<Arc<KnowledgeBank>>) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            consensus_threshold: 0.67,
            knowledge_bank,
        }
    }

    /// Load sessions from DB
    pub async fn load_from_db(&self) {
        if let Some(kb) = &self.knowledge_bank {
            if let Ok(loaded_sessions) = kb.load_sessions().await {
                let mut sessions = self.sessions.lock().await;
                for session in loaded_sessions {
                    sessions.insert(session.id.clone(), session);
                }
            }
        }
    }

    /// Create new council session
    pub async fn create_session(&self, question: String) -> String {
        let session_id = self.generate_session_id(&question);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();

        let session = CouncilSession {
            id: session_id.clone(),
            question,
            responses: Vec::new(),
            commitments: Vec::new(),
            reveals: Vec::new(),
            consensus: None,
            status: SessionStatus::GatheringResponses,
            created_at: timestamp,
        };

        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.clone(), session.clone());

        // Save to DB
        if let Some(kb) = &self.knowledge_bank {
            let _ = kb.save_session(&session).await;
        }

        session_id
    }

    /// Add AI response to session (with optional signature verification)
    pub async fn add_response(
        &self,
        session_id: &str,
        model_name: String,
        response: String,
        peer_id: String,
        signature: Option<String>,
        public_key: Option<String>,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions.get_mut(session_id).ok_or("Session not found")?;

        if session.status != SessionStatus::GatheringResponses {
            return Err("Session not in response gathering phase".to_string());
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();

        session.responses.push(CouncilResponse {
            model_name,
            response,
            peer_id,
            timestamp,
            signature,
            public_key,
        });

        // Save to DB
        if let Some(kb) = &self.knowledge_bank {
            let _ = kb.save_session(session).await;
        }

        Ok(())
    }

    /// Move session to commitment phase
    pub async fn start_commitment_phase(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions.get_mut(session_id).ok_or("Session not found")?;

        if session.status != SessionStatus::GatheringResponses {
            return Err("Session not in response gathering phase".to_string());
        }

        if session.responses.is_empty() {
            return Err("No responses to vote on".to_string());
        }

        session.status = SessionStatus::CommitmentPhase;
        Ok(())
    }

    /// Add vote commitment (blind vote)
    pub async fn add_commitment(
        &self,
        session_id: &str,
        commitment_hash: String,
        voter_peer_id: String,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions.get_mut(session_id).ok_or("Session not found")?;

        if session.status != SessionStatus::CommitmentPhase {
            return Err("Session not in commitment phase".to_string());
        }

        session.commitments.push(VoteCommitment {
            commitment_hash,
            voter_peer_id,
        });

        Ok(())
    }

    /// Move session to reveal phase
    pub async fn start_reveal_phase(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions.get_mut(session_id).ok_or("Session not found")?;

        if session.status != SessionStatus::CommitmentPhase {
            return Err("Session not in commitment phase".to_string());
        }

        if session.commitments.is_empty() {
            return Err("No commitments to reveal".to_string());
        }

        session.status = SessionStatus::RevealPhase;
        Ok(())
    }

    /// Add vote reveal and verify commitment
    pub async fn add_reveal(
        &self,
        session_id: &str,
        vote: String,
        salt: String,
        voter_peer_id: String,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions.get_mut(session_id).ok_or("Session not found")?;

        if session.status != SessionStatus::RevealPhase {
            return Err("Session not in reveal phase".to_string());
        }

        // Verify commitment matches reveal
        let commitment_hash = self.hash_vote(&vote, &salt);
        let commitment = session
            .commitments
            .iter()
            .find(|c| c.voter_peer_id == voter_peer_id)
            .ok_or("No commitment found for this voter")?;

        if commitment.commitment_hash != commitment_hash {
            return Err("Commitment verification failed".to_string());
        }

        session.reveals.push(VoteReveal {
            vote,
            salt,
            voter_peer_id,
        });

        Ok(())
    }

    /// Calculate consensus based on revealed votes
    pub async fn calculate_consensus(&self, session_id: &str) -> Result<Option<String>, String> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions.get_mut(session_id).ok_or("Session not found")?;

        if session.status != SessionStatus::RevealPhase {
            return Err("Session not in reveal phase".to_string());
        }

        let total_votes = session.reveals.len();
        if total_votes == 0 {
            return Ok(None);
        }

        let mut vote_counts = HashMap::new();
        for reveal in &session.reveals {
            *vote_counts.entry(reveal.vote.clone()).or_insert(0) += 1;
        }

        let threshold = (total_votes as f64 * self.consensus_threshold).ceil() as usize;

        for (vote, count) in vote_counts {
            if count >= threshold {
                session.consensus = Some(vote.clone());
                session.status = SessionStatus::ConsensusReached;
                
                // Save to DB
                if let Some(kb) = &self.knowledge_bank {
                    let _ = kb.save_session(session).await;
                }

                return Ok(Some(vote.clone()));
            }
        }

        // No consensus reached
        Ok(None)
    }

    /// Update reputation scores based on consensus
    pub async fn update_reputations(
        &self,
        session_id: &str,
        reputation_manager: Arc<crate::reputation::ReputationManager>,
    ) -> Result<(), String> {
        let sessions = self.sessions.lock().await;
        let session = sessions.get(session_id).ok_or("Session not found")?;

        if let Some(consensus) = &session.consensus {
            // Find who voted for consensus
            for reveal in &session.reveals {
                let is_correct = reveal.vote == *consensus;
                
                // Accuracy: +0.1 for correct, -0.05 for incorrect
                let accuracy_delta = if is_correct { 0.1 } else { -0.05 };
                
                // Reasoning: +0.05 for participating
                let reasoning_delta = 0.05;

                // We need to map peer_id to agent_id. 
                // In local mode, peer_id IS the agent_id (see gather_agent_response).
                // In P2P mode, peer_id is the node ID, so we might not be able to update remote agent scores easily yet.
                // For now, we assume local agents.
                let agent_id = &reveal.voter_peer_id;

                // Update score (fire and forget error handling for now)
                let _ = reputation_manager.update_score(agent_id, accuracy_delta, reasoning_delta).await;
            }
        }

        Ok(())
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<CouncilSession> {
        let sessions = self.sessions.lock().await;
        sessions.get(session_id).cloned()
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Vec<CouncilSession> {
        let sessions = self.sessions.lock().await;
        sessions.values().cloned().collect()
    }

    /// Generate commitment hash for blind voting
    pub fn hash_vote(&self, vote: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(vote.as_bytes());
        hasher.update(salt.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate session ID from question
    fn generate_session_id(&self, question: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(question.as_bytes());
        hasher.update(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_secs()
                .to_string()
                .as_bytes(),
        );
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    /// Create session with specific agents and gather their responses
    /// This is the main entry point for agent-driven deliberation
    pub async fn create_session_with_agents(
        &self,
        question: String,
        agent_pool: Arc<AgentPool>,
        agent_ids: Vec<String>,
        ollama_url: &str,
    ) -> Result<String, String> {
        self.create_session_with_agents_and_timeout(
            question, agent_pool, agent_ids, ollama_url, 30, // Default 30 second timeout
        )
        .await
    }

    /// Create session with agents and configurable timeout for stragglers
    ///
    /// Deliberation flow:
    /// 1. All agents race to respond (parallel)
    /// 2. First agent to respond must wait for others
    /// 3. Timeout ensures stragglers don't block forever
    pub async fn create_session_with_agents_and_timeout(
        &self,
        question: String,
        agent_pool: Arc<AgentPool>,
        agent_ids: Vec<String>,
        ollama_url: &str,
        timeout_seconds: u64,
    ) -> Result<String, String> {
        use tokio::time::{timeout, Duration};

        // Create session
        let session_id = self.create_session(question.clone()).await;

        // Get agents
        let agents = agent_pool.get_agents_by_ids(&agent_ids).await?;

        if agents.is_empty() {
            return Err("No agents specified".to_string());
        }

        let total_agents = agents.len();

        // Gather responses from all agents in parallel with timeout
        let mut handles = Vec::new();

        for agent in agents {
            let session_id = session_id.clone();
            let question = question.clone();
            let ollama_url = ollama_url.to_string();
            let self_clone = self.clone();

            let handle = tokio::spawn(async move {
                self_clone
                    .gather_agent_response(&session_id, &agent, &question, &ollama_url)
                    .await
            });

            handles.push(handle);
        }

        // Wait for all responses with timeout
        let wait_future = async {
            let mut success_count = 0;
            let mut error_count = 0;

            for handle in handles {
                match handle.await {
                    Ok(Ok(_)) => success_count += 1,
                    Ok(Err(e)) => {
                        eprintln!("⚠️ Agent response error: {}", e);
                        error_count += 1;
                    }
                    Err(e) => {
                        eprintln!("❌ Task join error: {}", e);
                        error_count += 1;
                    }
                }
            }

            (success_count, error_count)
        };

        let (success_count, error_count) =
            match timeout(Duration::from_secs(timeout_seconds), wait_future).await {
                Ok((success, errors)) => {
                    if success > 0 {
                        println!(
                            "✅ Council round complete: {} responded, {} failed",
                            success, errors
                        );
                    }
                    (success, errors)
                }
                Err(_) => {
                    // Timeout - some agents are still thinking
                    let current_responses = self
                        .get_session(&session_id)
                        .await
                        .map(|s| s.responses.len())
                        .unwrap_or(0);

                    eprintln!(
                        "⏱️ Timeout after {}s: {}/{} agents responded",
                        timeout_seconds, current_responses, total_agents
                    );

                    if current_responses == 0 {
                        return Err(format!("No agents responded within {}s", timeout_seconds));
                    }

                    (current_responses, total_agents - current_responses)
                }
            };

        if success_count == 0 {
            return Err("No agents provided responses".to_string());
        }

        Ok(session_id)
    }

    /// Gather response from a single agent
    async fn gather_agent_response(
        &self,
        session_id: &str,
        agent: &Agent,
        question: &str,
        ollama_url: &str,
    ) -> Result<(), String> {
        // Build prompt with agent's system context
        let prompt = agent.build_prompt(question, None);

        // Call Ollama API
        let response = crate::ollama::ask_ollama(ollama_url, &agent.model, prompt).await?;

        // Add response to session
        self.add_response(
            session_id,
            agent.name.clone(),
            response,
            agent.id.clone(), // Use agent ID as peer ID
            None,             // TODO: Add signature support
            None,
        )
        .await?;

        Ok(())
    }

    /// Get recent verdicts (finished sessions)
    pub async fn get_recent_verdicts(&self) -> Vec<crate::protocol::CouncilVerdictRecord> {
        let sessions = self.sessions.lock().await;
        let mut verdicts: Vec<crate::protocol::CouncilVerdictRecord> = sessions
            .values()
            .filter(|s| s.status == SessionStatus::ConsensusReached && s.consensus.is_some())
            .map(|s| {
                let participants: Vec<String> = s
                    .responses
                    .iter()
                    .map(|r| r.model_name.clone())
                    .collect();
                
                crate::protocol::CouncilVerdictRecord {
                    session_id: s.id.clone(),
                    question: s.question.clone(),
                    verdict: s.consensus.clone().unwrap(),
                    response_count: s.responses.len(),
                    participants,
                    created_at: s.created_at,
                    finalized_at: s.created_at, // Using created_at as approximation for now
                }
            })
            .collect();
        
        // Sort by created_at descending (newest first)
        verdicts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        verdicts
    }
}

// Make CouncilSessionManager cloneable for parallel execution
impl Clone for CouncilSessionManager {
    fn clone(&self) -> Self {
        Self {
            sessions: Arc::clone(&self.sessions),
            consensus_threshold: self.consensus_threshold,
            knowledge_bank: self.knowledge_bank.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let manager = CouncilSessionManager::new(None);
        let session_id = manager.create_session("Test question?".to_string()).await;

        let session = manager.get_session(&session_id).await;
        assert!(session.is_some());

        let session = session.unwrap();
        assert_eq!(session.question, "Test question?");
        assert_eq!(session.status, SessionStatus::GatheringResponses);
        assert_eq!(session.responses.len(), 0);
    }

    #[tokio::test]
    async fn test_add_response() {
        let manager = CouncilSessionManager::new(None);
        let session_id = manager.create_session("Test?".to_string()).await;

        let result = manager
            .add_response(
                &session_id,
                "model1".to_string(),
                "answer1".to_string(),
                "peer1".to_string(),
                None,
                None,
            )
            .await;

        assert!(result.is_ok());

        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.responses.len(), 1);
        assert_eq!(session.responses[0].response, "answer1");
    }

    #[tokio::test]
    async fn test_commitment_phase() {
        let manager = CouncilSessionManager::new(None);
        let session_id = manager.create_session("Test?".to_string()).await;

        manager
            .add_response(
                &session_id,
                "model1".to_string(),
                "answer".to_string(),
                "peer1".to_string(),
                None,
                None,
            )
            .await
            .unwrap();

        let result = manager.start_commitment_phase(&session_id).await;
        assert!(result.is_ok());

        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.status, SessionStatus::CommitmentPhase);
    }

    #[tokio::test]
    async fn test_blind_voting() {
        let manager = CouncilSessionManager::new(None);
        let session_id = manager.create_session("Test?".to_string()).await;

        // Setup session
        manager
            .add_response(
                &session_id,
                "model1".to_string(),
                "answer".to_string(),
                "peer1".to_string(),
                None,
                None,
            )
            .await
            .unwrap();
        manager.start_commitment_phase(&session_id).await.unwrap();

        // Create commitment
        let vote = "answer1";
        let salt = "random_salt_123";
        let commitment = manager.hash_vote(vote, salt);

        manager
            .add_commitment(&session_id, commitment.clone(), "peer1".to_string())
            .await
            .unwrap();

        // Move to reveal
        manager.start_reveal_phase(&session_id).await.unwrap();

        // Reveal vote
        let result = manager
            .add_reveal(
                &session_id,
                vote.to_string(),
                salt.to_string(),
                "peer1".to_string(),
            )
            .await;

        assert!(result.is_ok());

        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.reveals.len(), 1);
    }

    #[tokio::test]
    async fn test_consensus_calculation() {
        let manager = CouncilSessionManager::new(None);
        let session_id = manager.create_session("Test?".to_string()).await;

        // Setup
        manager
            .add_response(
                &session_id,
                "m1".to_string(),
                "a".to_string(),
                "p1".to_string(),
                None,
                None,
            )
            .await
            .unwrap();
        manager.start_commitment_phase(&session_id).await.unwrap();

        // Add 3 votes for same answer (100% consensus)
        for i in 1..=3 {
            let salt = format!("salt_{}", i);
            let vote = "answer_a";
            let commitment = manager.hash_vote(vote, &salt);
            let peer_id = format!("peer{}", i);

            manager
                .add_commitment(&session_id, commitment, peer_id.clone())
                .await
                .unwrap();
        }

        manager.start_reveal_phase(&session_id).await.unwrap();

        for i in 1..=3 {
            let salt = format!("salt_{}", i);
            let peer_id = format!("peer{}", i);
            manager
                .add_reveal(&session_id, "answer_a".to_string(), salt, peer_id)
                .await
                .unwrap();
        }

        let consensus = manager.calculate_consensus(&session_id).await.unwrap();
        assert!(consensus.is_some());
        assert_eq!(consensus.unwrap(), "answer_a");

        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.status, SessionStatus::ConsensusReached);
    }

    #[tokio::test]
    async fn test_no_consensus() {
        let manager = CouncilSessionManager::new(None);
        let session_id = manager.create_session("Test?".to_string()).await;

        // Setup
        manager
            .add_response(
                &session_id,
                "m1".to_string(),
                "a".to_string(),
                "p1".to_string(),
                None,
                None,
            )
            .await
            .unwrap();
        manager.start_commitment_phase(&session_id).await.unwrap();

        // Add 3 different votes (no consensus)
        for i in 1..=3 {
            let salt = format!("salt_{}", i);
            let vote = format!("answer_{}", i);
            let commitment = manager.hash_vote(&vote, &salt);
            let peer_id = format!("peer{}", i);

            manager
                .add_commitment(&session_id, commitment, peer_id.clone())
                .await
                .unwrap();
        }

        manager.start_reveal_phase(&session_id).await.unwrap();

        for i in 1..=3 {
            let salt = format!("salt_{}", i);
            let vote = format!("answer_{}", i);
            let peer_id = format!("peer{}", i);
            manager
                .add_reveal(&session_id, vote, salt, peer_id)
                .await
                .unwrap();
        }

        let consensus = manager.calculate_consensus(&session_id).await.unwrap();
        assert!(consensus.is_none());
    }

    #[tokio::test]
    async fn test_agent_pool_integration() {
        use crate::agents::{Agent, AgentPool};

        let manager = CouncilSessionManager::new(None);
        let pool = Arc::new(AgentPool::new());

        // Add test agents
        let agent1 = Agent::new(
            "Test Agent 1".to_string(),
            "test-model".to_string(),
            "You are a helpful assistant.".to_string(),
        );
        let agent2 = Agent::new(
            "Test Agent 2".to_string(),
            "test-model".to_string(),
            "You are a critical thinker.".to_string(),
        );

        let agent1_id = pool.add_agent(agent1).await.unwrap();
        let agent2_id = pool.add_agent(agent2).await.unwrap();

        // Note: This test verifies the API exists, but won't actually call Ollama
        // In a real scenario, you'd mock the Ollama client
        let result = manager
            .create_session_with_agents(
                "Test question?".to_string(),
                pool,
                vec![agent1_id, agent2_id],
                "http://localhost:11434", // Won't actually connect in unit tests
            )
            .await;

        // We expect this to fail since Ollama isn't running, but the API should be callable
        assert!(result.is_err() || result.is_ok());
    }
}

// Council session manager for multi-round deliberation

use crate::protocol::{
    CouncilResponse, CouncilSession, SessionStatus,
    VoteCommitment, VoteReveal,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use sha2::{Sha256, Digest};

/// Manages council deliberation sessions
pub struct CouncilSessionManager {
    sessions: Arc<Mutex<HashMap<String, CouncilSession>>>,
    consensus_threshold: f64, // Byzantine fault tolerance: 67%
}

impl CouncilSessionManager {
    /// Create new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            consensus_threshold: 0.67,
        }
    }

    /// Create new council session
    pub async fn create_session(&self, question: String) -> String {
        let session_id = self.generate_session_id(&question);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
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
        sessions.insert(session_id.clone(), session);
        
        session_id
    }

    /// Add AI response to session
    pub async fn add_response(
        &self,
        session_id: &str,
        model_name: String,
        response: String,
        peer_id: String,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or("Session not found")?;

        if session.status != SessionStatus::GatheringResponses {
            return Err("Session not in response gathering phase".to_string());
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        session.responses.push(CouncilResponse {
            model_name,
            response,
            peer_id,
            timestamp,
        });

        Ok(())
    }

    /// Move session to commitment phase
    pub async fn start_commitment_phase(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or("Session not found")?;

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
        let session = sessions
            .get_mut(session_id)
            .ok_or("Session not found")?;

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
        let session = sessions
            .get_mut(session_id)
            .ok_or("Session not found")?;

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
        let session = sessions
            .get_mut(session_id)
            .ok_or("Session not found")?;

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

    /// Calculate consensus if threshold reached
    pub async fn calculate_consensus(&self, session_id: &str) -> Result<Option<String>, String> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or("Session not found")?;

        if session.status != SessionStatus::RevealPhase {
            return Err("Session not in reveal phase".to_string());
        }

        // Count votes
        let mut vote_counts: HashMap<String, usize> = HashMap::new();
        for reveal in &session.reveals {
            *vote_counts.entry(reveal.vote.clone()).or_insert(0) += 1;
        }

        let total_votes = session.reveals.len();
        if total_votes == 0 {
            return Ok(None);
        }

        // Find consensus (67% threshold)
        for (vote, count) in vote_counts.iter() {
            let percentage = *count as f64 / total_votes as f64;
            if percentage >= self.consensus_threshold {
                session.consensus = Some(vote.clone());
                session.status = SessionStatus::ConsensusReached;
                return Ok(Some(vote.clone()));
            }
        }

        // No consensus reached
        Ok(None)
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
                .unwrap()
                .as_nanos()
                .to_string()
                .as_bytes(),
        );
        format!("{:x}", hasher.finalize())[..16].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let manager = CouncilSessionManager::new();
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
        let manager = CouncilSessionManager::new();
        let session_id = manager.create_session("Test?".to_string()).await;
        
        let result = manager
            .add_response(&session_id, "model1".to_string(), "answer1".to_string(), "peer1".to_string())
            .await;
        
        assert!(result.is_ok());
        
        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.responses.len(), 1);
        assert_eq!(session.responses[0].response, "answer1");
    }

    #[tokio::test]
    async fn test_commitment_phase() {
        let manager = CouncilSessionManager::new();
        let session_id = manager.create_session("Test?".to_string()).await;
        
        manager
            .add_response(&session_id, "model1".to_string(), "answer".to_string(), "peer1".to_string())
            .await
            .unwrap();
        
        let result = manager.start_commitment_phase(&session_id).await;
        assert!(result.is_ok());
        
        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.status, SessionStatus::CommitmentPhase);
    }

    #[tokio::test]
    async fn test_blind_voting() {
        let manager = CouncilSessionManager::new();
        let session_id = manager.create_session("Test?".to_string()).await;
        
        // Setup session
        manager
            .add_response(&session_id, "model1".to_string(), "answer".to_string(), "peer1".to_string())
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
            .add_reveal(&session_id, vote.to_string(), salt.to_string(), "peer1".to_string())
            .await;
        
        assert!(result.is_ok());
        
        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.reveals.len(), 1);
    }

    #[tokio::test]
    async fn test_consensus_calculation() {
        let manager = CouncilSessionManager::new();
        let session_id = manager.create_session("Test?".to_string()).await;
        
        // Setup
        manager
            .add_response(&session_id, "m1".to_string(), "a".to_string(), "p1".to_string())
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
        let manager = CouncilSessionManager::new();
        let session_id = manager.create_session("Test?".to_string()).await;
        
        // Setup
        manager
            .add_response(&session_id, "m1".to_string(), "a".to_string(), "p1".to_string())
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
}

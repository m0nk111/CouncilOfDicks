// Council message protocol for P2P communication

use crate::crypto::SignedMessage;
use crate::reputation::AgentReputation;
use serde::{Deserialize, Serialize};

/// Message types for council communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CouncilMessage {
    /// Initial question to deliberate
    Question {
        id: String,
        question: String,
        requester_peer_id: String,
    },

    /// AI model response to question (cryptographically signed)
    Response {
        question_id: String,
        model_name: String,
        signed_response: SignedMessage, // Content is signed, immutable
        peer_id: String,
        reputation: Option<AgentReputation>, // Piggyback reputation
    },

    /// Blind vote commitment (hash of actual vote)
    VoteCommitment {
        question_id: String,
        commitment_hash: String,
        voter_peer_id: String,
    },

    /// Reveal actual vote after all commitments received
    VoteReveal {
        question_id: String,
        vote: String,
        salt: String, // For verifying commitment
        voter_peer_id: String,
    },

    /// Announce consensus reached
    ConsensusReached {
        question_id: String,
        final_answer: String,
        vote_count: u32,
        participating_peers: Vec<String>,
    },

    /// Heartbeat to prove human is active
    Heartbeat {
        peer_id: String,
        timestamp: u64,
    },

    /// Update the active topic for the network
    TopicUpdate {
        topic: String,
        interval: u64,
        set_by_peer_id: String,
        timestamp: u64,
    },

    /// Request peer to prove human presence
    HumanChallenge {
        peer_id: String,
        challenge: String,
        expires_at: u64,
    },

    /// Peer discovery announcement
    PeerAnnouncement {
        peer_id: String,
        models: Vec<String>,
        reputation_tier: String,
    },
    
    /// Sync reputation scores
    ReputationSync {
        peer_id: String,
        reputation: AgentReputation,
    },
}

impl CouncilMessage {
    /// Serialize message to JSON bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize message from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Get message type as string
    pub fn message_type(&self) -> &str {
        match self {
            CouncilMessage::Question { .. } => "Question",
            CouncilMessage::Response { .. } => "Response",
            CouncilMessage::VoteCommitment { .. } => "VoteCommitment",
            CouncilMessage::VoteReveal { .. } => "VoteReveal",
            CouncilMessage::ConsensusReached { .. } => "ConsensusReached",
            CouncilMessage::Heartbeat { .. } => "Heartbeat",
            CouncilMessage::HumanChallenge { .. } => "HumanChallenge",
            CouncilMessage::PeerAnnouncement { .. } => "PeerAnnouncement",
            CouncilMessage::TopicUpdate { .. } => "TopicUpdate",
            CouncilMessage::ReputationSync { .. } => "ReputationSync",
        }
    }
}

/// Council session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilSession {
    pub id: String,
    pub question: String,
    pub responses: Vec<CouncilResponse>,
    pub commitments: Vec<VoteCommitment>,
    pub reveals: Vec<VoteReveal>,
    pub consensus: Option<String>,
    pub status: SessionStatus,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilResponse {
    pub model_name: String,
    pub response: String,
    pub peer_id: String,
    pub timestamp: u64,
    pub signature: Option<String>,  // Base64 encoded Ed25519 signature
    pub public_key: Option<String>, // Base64 encoded public key
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteCommitment {
    pub commitment_hash: String,
    pub voter_peer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteReveal {
    pub vote: String,
    pub salt: String,
    pub voter_peer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    GatheringResponses,
    CommitmentPhase,
    RevealPhase,
    ConsensusReached,
    Failed,
}

/// Record of a finalized council verdict for history/UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilVerdictRecord {
    pub session_id: String,
    pub question: String,
    pub verdict: String,
    pub response_count: usize,
    pub participants: Vec<String>,
    pub created_at: u64,
    pub finalized_at: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = CouncilMessage::Question {
            id: "test-123".to_string(),
            question: "What is the answer?".to_string(),
            requester_peer_id: "peer-xyz".to_string(),
        };

        let bytes = msg.to_bytes().unwrap();
        let deserialized = CouncilMessage::from_bytes(&bytes).unwrap();

        assert_eq!(msg.message_type(), deserialized.message_type());
    }

    #[test]
    fn test_message_types() {
        let question = CouncilMessage::Question {
            id: "1".to_string(),
            question: "test".to_string(),
            requester_peer_id: "peer1".to_string(),
        };
        assert_eq!(question.message_type(), "Question");

        let heartbeat = CouncilMessage::Heartbeat {
            peer_id: "peer1".to_string(),
            timestamp: 123456,
        };
        assert_eq!(heartbeat.message_type(), "Heartbeat");
    }

    #[test]
    fn test_session_status() {
        let session = CouncilSession {
            id: "test".to_string(),
            question: "test".to_string(),
            responses: vec![],
            commitments: vec![],
            reveals: vec![],
            consensus: None,
            status: SessionStatus::GatheringResponses,
            created_at: 0,
        };

        assert_eq!(session.status, SessionStatus::GatheringResponses);
    }
}

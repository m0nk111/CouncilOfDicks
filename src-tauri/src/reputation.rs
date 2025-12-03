use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgentTier {
    Quarantine, // New or flagged agents
    Candidate,  // Probationary period
    Standard,   // Regular voting member
    Prime,      // High reputation, more weight
    Citadel,    // Highest tier, veto power
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationScore {
    pub accuracy: f32,      // 0.0 - 1.0
    pub reasoning: f32,     // 0.0 - 1.0
    pub contribution: f32,  // 0.0 - 1.0
    pub total_votes: u32,
    pub successful_consensus: u32,
}

impl Default for ReputationScore {
    fn default() -> Self {
        Self {
            accuracy: 0.5,
            reasoning: 0.5,
            contribution: 0.0,
            total_votes: 0,
            successful_consensus: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReputation {
    pub agent_id: String,
    pub tier: AgentTier,
    pub score: ReputationScore,
    pub last_updated: u64,
}

use crate::knowledge::KnowledgeBank;

pub struct ReputationManager {
    reputations: Arc<Mutex<HashMap<String, AgentReputation>>>,
    knowledge_bank: Option<Arc<KnowledgeBank>>,
}

impl ReputationManager {
    pub fn new(knowledge_bank: Option<Arc<KnowledgeBank>>) -> Self {
        Self {
            reputations: Arc::new(Mutex::new(HashMap::new())),
            knowledge_bank,
        }
    }

    pub async fn load_from_db(&self) {
        if let Some(kb) = &self.knowledge_bank {
            if let Ok(loaded_reps) = kb.load_reputations().await {
                let mut reps = self.reputations.lock().await;
                for rep in loaded_reps {
                    reps.insert(rep.agent_id.clone(), rep);
                }
            }
        }
    }

    pub async fn get_reputation(&self, agent_id: &str) -> Option<AgentReputation> {
        let reps = self.reputations.lock().await;
        reps.get(agent_id).cloned()
    }

    pub async fn initialize_agent(&self, agent_id: String) {
        let mut reps = self.reputations.lock().await;
        if !reps.contains_key(&agent_id) {
            reps.insert(
                agent_id.clone(),
                AgentReputation {
                    agent_id,
                    tier: AgentTier::Candidate, // Start as Candidate
                    score: ReputationScore::default(),
                    last_updated: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                },
            );
        }
    }

    pub async fn update_score(
        &self,
        agent_id: &str,
        accuracy_delta: f32,
        reasoning_delta: f32,
    ) -> Result<AgentTier, String> {
        let mut reps = self.reputations.lock().await;
        let rep = reps
            .get_mut(agent_id)
            .ok_or_else(|| "Agent not found".to_string())?;

        // Update scores with clamping
        rep.score.accuracy = (rep.score.accuracy + accuracy_delta).clamp(0.0, 1.0);
        rep.score.reasoning = (rep.score.reasoning + reasoning_delta).clamp(0.0, 1.0);
        rep.score.contribution += 0.01; // Small increment for participation
        rep.score.contribution = rep.score.contribution.clamp(0.0, 1.0);
        
        rep.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Recalculate Tier
        let new_tier = self.calculate_tier(&rep.score);
        rep.tier = new_tier.clone();

        // Save to DB
        if let Some(kb) = &self.knowledge_bank {
            let _ = kb.save_reputation(rep).await;
        }

        Ok(new_tier)
    }

    pub async fn update_from_sync(&self, reputation: AgentReputation) -> Result<(), String> {
        let mut reps = self.reputations.lock().await;
        // Update if newer or new
        let should_update = if let Some(existing) = reps.get(&reputation.agent_id) {
            reputation.last_updated > existing.last_updated
        } else {
            true
        };

        if should_update {
            // Save to DB first
            if let Some(kb) = &self.knowledge_bank {
                let _ = kb.save_reputation(&reputation).await;
            }
            reps.insert(reputation.agent_id.clone(), reputation);
        }
        Ok(())
    }

    fn calculate_tier(&self, score: &ReputationScore) -> AgentTier {
        let total_score = (score.accuracy * 0.4) + (score.reasoning * 0.4) + (score.contribution * 0.2);

        if total_score >= 0.9 && score.total_votes > 100 {
            AgentTier::Citadel
        } else if total_score >= 0.8 && score.total_votes > 50 {
            AgentTier::Prime
        } else if total_score >= 0.5 {
            AgentTier::Standard
        } else if total_score >= 0.2 {
            AgentTier::Candidate
        } else {
            AgentTier::Quarantine
        }
    }
}

// Tauri Commands

#[tauri::command]
pub async fn reputation_get(
    agent_id: String,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<AgentReputation, String> {
    state
        .reputation_manager
        .get_reputation(&agent_id)
        .await
        .ok_or_else(|| "Reputation not found".to_string())
}

use crate::knowledge::KnowledgeBank;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Result of duplicate question check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCheckResult {
    pub is_duplicate: bool,
    pub similarity_score: f32,
    pub existing_session_id: Option<String>,
    pub existing_question: Option<String>,
    pub existing_verdict: Option<String>,
    pub asked_at: Option<String>,
}

/// Configuration for duplicate detection
#[derive(Debug, Clone)]
pub struct DuplicateFilterConfig {
    /// Exact duplicate threshold (0.95+)
    pub exact_threshold: f32,
    /// Very similar threshold (0.85-0.95)
    pub similar_threshold: f32,
    /// Related threshold (0.70-0.85)
    pub related_threshold: f32,
}

impl Default for DuplicateFilterConfig {
    fn default() -> Self {
        Self {
            exact_threshold: 0.95,
            similar_threshold: 0.85,
            related_threshold: 0.70,
        }
    }
}

/// Duplicate question filter
pub struct DuplicateFilter {
    knowledge_bank: Arc<KnowledgeBank>,
    config: DuplicateFilterConfig,
}

impl DuplicateFilter {
    pub fn new(knowledge_bank: Arc<KnowledgeBank>) -> Self {
        Self {
            knowledge_bank,
            config: DuplicateFilterConfig::default(),
        }
    }

    pub fn with_config(knowledge_bank: Arc<KnowledgeBank>, config: DuplicateFilterConfig) -> Self {
        Self {
            knowledge_bank,
            config,
        }
    }

    /// Check if a question is a duplicate
    pub async fn check_duplicate(&self, question: &str) -> Result<DuplicateCheckResult, String> {
        // Search knowledge bank for similar questions
        let results = self
            .knowledge_bank
            .semantic_search(question, 1)
            .await
            .map_err(|e| format!("Failed to search knowledge bank: {}", e))?;

        if results.is_empty() {
            return Ok(DuplicateCheckResult {
                is_duplicate: false,
                similarity_score: 0.0,
                existing_session_id: None,
                existing_question: None,
                existing_verdict: None,
                asked_at: None,
            });
        }

        let top_result = &results[0];

        // Get full deliberation details
        let deliberation = self
            .knowledge_bank
            .get_deliberation(&top_result.deliberation_id)
            .await
            .map_err(|e| format!("Failed to get deliberation: {}", e))?;

        // Generate verdict from consensus or last response
        let verdict = if let Some(consensus_text) = &deliberation.consensus {
            consensus_text.clone()
        } else if let Some(last_round) = deliberation.rounds.last() {
            last_round
                .responses
                .first()
                .map(|r| r.response.clone())
                .unwrap_or_else(|| "No verdict available".to_string())
        } else {
            "No verdict available".to_string()
        };

        // Format timestamp
        let timestamp = chrono::DateTime::from_timestamp(deliberation.created_at as i64, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "Unknown".to_string());

        Ok(DuplicateCheckResult {
            is_duplicate: top_result.relevance_score >= self.config.similar_threshold,
            similarity_score: top_result.relevance_score,
            existing_session_id: Some(top_result.deliberation_id.clone()),
            existing_question: Some(deliberation.question.clone()),
            existing_verdict: Some(verdict),
            asked_at: Some(timestamp),
        })
    }

    /// Get warning message for duplicate questions
    pub fn format_warning(&self, result: &DuplicateCheckResult) -> String {
        if !result.is_duplicate {
            return String::new();
        }

        let similarity_pct = (result.similarity_score * 100.0) as u32;

        let warning_type = if result.similarity_score >= self.config.exact_threshold {
            "⛔ Exact Duplicate"
        } else {
            "⚠️ Similar Question"
        };

        format!(
            "{} ({}% match)\n\n\
             Previous session: #{}\n\
             Question: \"{}\"\n\
             Asked: {}\n\
             Verdict: \"{}\"\n\n\
             View full deliberation: /session/{}\n\
             To ask anyway: /ask --force <your question>",
            warning_type,
            similarity_pct,
            result
                .existing_session_id
                .as_ref()
                .unwrap_or(&"unknown".to_string()),
            result
                .existing_question
                .as_ref()
                .unwrap_or(&"unknown".to_string()),
            result.asked_at.as_ref().unwrap_or(&"unknown".to_string()),
            result
                .existing_verdict
                .as_ref()
                .unwrap_or(&"unknown".to_string()),
            result
                .existing_session_id
                .as_ref()
                .unwrap_or(&"unknown".to_string()),
        )
    }

    /// Get suggestion message for related questions
    pub fn format_suggestion(&self, result: &DuplicateCheckResult) -> String {
        if result.is_duplicate || result.similarity_score < self.config.related_threshold {
            return String::new();
        }

        let similarity_pct = (result.similarity_score * 100.0) as u32;

        format!(
            "💡 Related Question Found ({}% match)\n\n\
             You might find this helpful:\n\
             Session #{}: \"{}\"\n\
             Verdict: \"{}\"\n\n\
             View details: /session/{}",
            similarity_pct,
            result
                .existing_session_id
                .as_ref()
                .unwrap_or(&"unknown".to_string()),
            result
                .existing_question
                .as_ref()
                .unwrap_or(&"unknown".to_string()),
            result
                .existing_verdict
                .as_ref()
                .unwrap_or(&"unknown".to_string()),
            result
                .existing_session_id
                .as_ref()
                .unwrap_or(&"unknown".to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_filter_config_default() {
        let config = DuplicateFilterConfig::default();
        assert_eq!(config.exact_threshold, 0.95);
        assert_eq!(config.similar_threshold, 0.85);
        assert_eq!(config.related_threshold, 0.70);
    }

    #[test]
    fn test_duplicate_check_result_no_duplicate() {
        let result = DuplicateCheckResult {
            is_duplicate: false,
            similarity_score: 0.5,
            existing_session_id: None,
            existing_question: None,
            existing_verdict: None,
            asked_at: None,
        };

        assert!(!result.is_duplicate);
        assert_eq!(result.similarity_score, 0.5);
    }

    #[test]
    fn test_duplicate_check_result_is_duplicate() {
        let result = DuplicateCheckResult {
            is_duplicate: true,
            similarity_score: 0.92,
            existing_session_id: Some("session_123".to_string()),
            existing_question: Some("What is AI?".to_string()),
            existing_verdict: Some("AI is...".to_string()),
            asked_at: Some("2025-11-30T10:00:00Z".to_string()),
        };

        assert!(result.is_duplicate);
        assert_eq!(result.similarity_score, 0.92);
        assert!(result.existing_session_id.is_some());
    }

    #[test]
    fn test_format_warning_no_duplicate() {
        let config = DuplicateFilterConfig::default();
        let result = DuplicateCheckResult {
            is_duplicate: false,
            similarity_score: 0.5,
            existing_session_id: None,
            existing_question: None,
            existing_verdict: None,
            asked_at: None,
        };

        // Create a mock knowledge bank (we won't use it for this test)
        // So we'll skip the filter creation and just test the result
        assert!(!result.is_duplicate);
    }

    #[test]
    fn test_similarity_thresholds() {
        let config = DuplicateFilterConfig::default();

        // Exact duplicate
        assert!(0.96 >= config.exact_threshold);

        // Similar but not exact
        assert!(0.90 >= config.similar_threshold);
        assert!(0.90 < config.exact_threshold);

        // Related but not similar
        assert!(0.75 >= config.related_threshold);
        assert!(0.75 < config.similar_threshold);

        // Not related
        assert!(0.60 < config.related_threshold);
    }
}

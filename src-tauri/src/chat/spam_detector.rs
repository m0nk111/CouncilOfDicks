use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Spam detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamDetectorConfig {
    pub duplicate_window_seconds: u64,
    pub rapid_fire_threshold: usize,
    pub rapid_fire_window_seconds: u64,
    pub min_message_length: usize,
    pub all_caps_ratio_threshold: f32,
    pub spam_keywords: Vec<String>,
}

impl Default for SpamDetectorConfig {
    fn default() -> Self {
        Self {
            duplicate_window_seconds: 60,
            rapid_fire_threshold: 5,
            rapid_fire_window_seconds: 10,
            min_message_length: 5,
            all_caps_ratio_threshold: 0.8,
            spam_keywords: vec![
                "buy now".to_string(),
                "click here".to_string(),
                "limited offer".to_string(),
                "act now".to_string(),
                "guaranteed".to_string(),
                "free money".to_string(),
            ],
        }
    }
}

/// Spam score levels and actions
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SpamLevel {
    Ok,         // 0.0-0.3
    Warning,    // 0.3-0.5
    Cooldown5m, // 0.5-0.7
    Cooldown1h, // 0.7-0.9
    Ban24h,     // 0.9-1.0
}

impl SpamLevel {
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s < 0.3 => SpamLevel::Ok,
            s if s < 0.5 => SpamLevel::Warning,
            s if s < 0.7 => SpamLevel::Cooldown5m,
            s if s < 0.9 => SpamLevel::Cooldown1h,
            _ => SpamLevel::Ban24h,
        }
    }

    pub fn cooldown_seconds(&self) -> Option<i64> {
        match self {
            SpamLevel::Ok | SpamLevel::Warning => None,
            SpamLevel::Cooldown5m => Some(300),
            SpamLevel::Cooldown1h => Some(3600),
            SpamLevel::Ban24h => Some(86400),
        }
    }
}

/// User spam tracking
#[derive(Debug, Clone)]
struct UserSpamState {
    messages: Vec<(DateTime<Utc>, String)>,
    spam_score: f32,
    banned_until: Option<DateTime<Utc>>,
}

impl UserSpamState {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            spam_score: 0.0,
            banned_until: None,
        }
    }

    fn cleanup(&mut self, now: DateTime<Utc>) {
        // Keep only last hour
        self.messages
            .retain(|(ts, _)| now.signed_duration_since(*ts) < Duration::hours(1));
    }

    fn is_banned(&self, now: DateTime<Utc>) -> bool {
        if let Some(until) = self.banned_until {
            now < until
        } else {
            false
        }
    }
}

/// Spam detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamCheckResult {
    pub is_spam: bool,
    pub spam_score: f32,
    pub spam_level: SpamLevel,
    pub reasons: Vec<String>,
    pub cooldown_seconds: Option<i64>,
}

/// Spam detector for message content
pub struct SpamDetector {
    config: SpamDetectorConfig,
    users: Arc<Mutex<HashMap<String, UserSpamState>>>,
}

impl SpamDetector {
    pub fn new() -> Self {
        Self {
            config: SpamDetectorConfig::default(),
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_config(config: SpamDetectorConfig) -> Self {
        Self {
            config,
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if message is spam
    pub fn check_spam(&self, user_id: &str, message: &str) -> SpamCheckResult {
        let mut users = self.users.lock().unwrap();
        let now = Utc::now();

        let user_state = users
            .entry(user_id.to_string())
            .or_insert_with(UserSpamState::new);
        user_state.cleanup(now);

        // Check if banned
        if user_state.is_banned(now) {
            let cooldown = user_state
                .banned_until
                .unwrap()
                .signed_duration_since(now)
                .num_seconds();
            return SpamCheckResult {
                is_spam: true,
                spam_score: 1.0,
                spam_level: SpamLevel::Ban24h,
                reasons: vec!["User is banned".to_string()],
                cooldown_seconds: Some(cooldown),
            };
        }

        let mut score = 0.0;
        let mut reasons = Vec::new();

        // Check duplicate messages
        if self.has_duplicate_in_window(user_state, message) {
            score += 0.3;
            reasons.push("Duplicate message in short time window".to_string());
        }

        // Check rapid fire
        let rapid_fire_count = self.count_rapid_fire(user_state, now);
        if rapid_fire_count >= self.config.rapid_fire_threshold {
            score += 0.4;
            reasons.push(format!(
                "Rapid-fire detected: {} messages in {}s",
                rapid_fire_count, self.config.rapid_fire_window_seconds
            ));
        }

        // Check message length
        if message.trim().len() < self.config.min_message_length {
            score += 0.2;
            reasons.push(format!(
                "Message too short (< {} chars)",
                self.config.min_message_length
            ));
        }

        // Check ALL CAPS
        if self.is_all_caps(message) {
            score += 0.2;
            reasons.push("Excessive caps lock usage".to_string());
        }

        // Check spam keywords
        if self.contains_spam_keywords(message) {
            score += 0.5;
            reasons.push("Contains spam keywords".to_string());
        }

        // Determine spam level
        let spam_level = SpamLevel::from_score(score);
        let is_spam = matches!(
            spam_level,
            SpamLevel::Cooldown5m | SpamLevel::Cooldown1h | SpamLevel::Ban24h
        );

        // Apply cooldown if needed
        if let Some(cooldown_seconds) = spam_level.cooldown_seconds() {
            user_state.banned_until = Some(now + Duration::seconds(cooldown_seconds));
        }

        // Update spam score (exponential moving average)
        user_state.spam_score = user_state.spam_score * 0.7 + score * 0.3;

        SpamCheckResult {
            is_spam,
            spam_score: score,
            spam_level,
            reasons,
            cooldown_seconds: spam_level.cooldown_seconds(),
        }
    }

    /// Record message
    pub fn record_message(&self, user_id: &str, message: &str) {
        let mut users = self.users.lock().unwrap();
        let now = Utc::now();

        let user_state = users
            .entry(user_id.to_string())
            .or_insert_with(UserSpamState::new);
        user_state.messages.push((now, message.to_string()));
    }

    /// Check for duplicate messages in time window
    fn has_duplicate_in_window(&self, user_state: &UserSpamState, message: &str) -> bool {
        let now = Utc::now();
        let window = Duration::seconds(self.config.duplicate_window_seconds as i64);

        user_state.messages.iter().any(|(ts, msg)| {
            now.signed_duration_since(*ts) < window && msg.trim() == message.trim()
        })
    }

    /// Count messages in rapid-fire window
    fn count_rapid_fire(&self, user_state: &UserSpamState, now: DateTime<Utc>) -> usize {
        let window = Duration::seconds(self.config.rapid_fire_window_seconds as i64);
        user_state
            .messages
            .iter()
            .filter(|(ts, _)| now.signed_duration_since(*ts) < window)
            .count()
    }

    /// Check if message is mostly caps
    fn is_all_caps(&self, message: &str) -> bool {
        let letters: Vec<char> = message.chars().filter(|c| c.is_alphabetic()).collect();
        if letters.is_empty() {
            return false;
        }

        let caps_count = letters.iter().filter(|c| c.is_uppercase()).count();
        let ratio = caps_count as f32 / letters.len() as f32;

        ratio >= self.config.all_caps_ratio_threshold
    }

    /// Check for spam keywords
    fn contains_spam_keywords(&self, message: &str) -> bool {
        let lower = message.to_lowercase();
        self.config
            .spam_keywords
            .iter()
            .any(|keyword| lower.contains(keyword))
    }

    /// Reset user state
    pub fn reset_user(&self, user_id: &str) {
        let mut users = self.users.lock().unwrap();
        users.remove(user_id);
    }
}

impl Default for SpamDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spam_level_from_score() {
        assert_eq!(SpamLevel::from_score(0.2), SpamLevel::Ok);
        assert_eq!(SpamLevel::from_score(0.4), SpamLevel::Warning);
        assert_eq!(SpamLevel::from_score(0.6), SpamLevel::Cooldown5m);
        assert_eq!(SpamLevel::from_score(0.8), SpamLevel::Cooldown1h);
        assert_eq!(SpamLevel::from_score(0.95), SpamLevel::Ban24h);
    }

    #[test]
    fn test_spam_level_cooldown() {
        assert_eq!(SpamLevel::Ok.cooldown_seconds(), None);
        assert_eq!(SpamLevel::Cooldown5m.cooldown_seconds(), Some(300));
        assert_eq!(SpamLevel::Cooldown1h.cooldown_seconds(), Some(3600));
        assert_eq!(SpamLevel::Ban24h.cooldown_seconds(), Some(86400));
    }

    #[test]
    fn test_detector_allows_normal_message() {
        let detector = SpamDetector::new();
        let result = detector.check_spam("user1", "This is a normal message");

        assert!(!result.is_spam);
        assert_eq!(result.spam_level, SpamLevel::Ok);
    }

    #[test]
    fn test_detector_catches_duplicate() {
        let detector = SpamDetector::new();

        detector.record_message("user1", "same message");
        let result = detector.check_spam("user1", "same message");

        assert!(result.spam_score > 0.0);
        assert!(result.reasons.iter().any(|r| r.contains("Duplicate")));
    }

    #[test]
    fn test_detector_catches_short_message() {
        let detector = SpamDetector::new();
        let result = detector.check_spam("user1", "Hi");

        assert!(result.spam_score > 0.0);
        assert!(result.reasons.iter().any(|r| r.contains("too short")));
    }

    #[test]
    fn test_detector_catches_all_caps() {
        let detector = SpamDetector::new();
        let result = detector.check_spam("user1", "THIS IS ALL CAPS MESSAGE");

        assert!(result.spam_score > 0.0);
        assert!(result.reasons.iter().any(|r| r.contains("caps")));
    }

    #[test]
    fn test_detector_catches_spam_keywords() {
        let detector = SpamDetector::new();
        let result = detector.check_spam("user1", "Click here for free money!");

        assert!(result.spam_score > 0.0);
        assert!(result.reasons.iter().any(|r| r.contains("spam keywords")));
    }

    #[test]
    fn test_detector_rapid_fire() {
        let detector = SpamDetector::new();

        for i in 0..6 {
            detector.record_message("user1", &format!("Message {}", i));
        }

        let result = detector.check_spam("user1", "Another message");
        assert!(result.spam_score > 0.0);
        assert!(result.reasons.iter().any(|r| r.contains("Rapid-fire")));
    }

    #[test]
    fn test_detector_reset_user() {
        let detector = SpamDetector::new();

        detector.record_message("user1", "spam");
        detector.record_message("user1", "spam");
        detector.record_message("user1", "spam");

        let result1 = detector.check_spam("user1", "spam");
        assert!(result1.spam_score > 0.0);

        detector.reset_user("user1");
        let result2 = detector.check_spam("user1", "normal message");
        assert!(!result2.is_spam);
    }

    #[test]
    fn test_user_spam_state_cleanup() {
        let mut state = UserSpamState::new();
        let now = Utc::now();

        state
            .messages
            .push((now - Duration::hours(2), "old".to_string()));
        state
            .messages
            .push((now - Duration::minutes(5), "recent".to_string()));

        state.cleanup(now);

        assert_eq!(state.messages.len(), 1);
        assert_eq!(state.messages[0].1, "recent");
    }

    #[test]
    fn test_banned_user() {
        let detector = SpamDetector::new();

        // Trigger high spam score to get banned
        for _ in 0..10 {
            detector.record_message("user1", "spam");
        }

        let result = detector.check_spam("user1", "buy now click here");

        if result.is_spam && result.cooldown_seconds.is_some() {
            // Now check if banned
            let result2 = detector.check_spam("user1", "normal message");
            assert!(result2.is_spam);
            assert!(result2.reasons.iter().any(|r| r.contains("banned")));
        }
    }
}

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub max_questions_per_minute: usize,
    pub max_questions_per_hour: usize,
    pub max_questions_per_day: usize,
    pub initial_cooldown_seconds: u64,
    pub max_cooldown_seconds: u64,
    pub cooldown_multiplier: f32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_questions_per_minute: 2,
            max_questions_per_hour: 10,
            max_questions_per_day: 50,
            initial_cooldown_seconds: 30,
            max_cooldown_seconds: 3600,
            cooldown_multiplier: 2.0,
        }
    }
}

/// User's rate limit state
#[derive(Debug, Clone)]
struct UserState {
    questions: Vec<DateTime<Utc>>,
    violations: usize,
    cooldown_until: Option<DateTime<Utc>>,
}

impl UserState {
    fn new() -> Self {
        Self {
            questions: Vec::new(),
            violations: 0,
            cooldown_until: None,
        }
    }

    /// Clean old timestamps
    fn cleanup(&mut self, now: DateTime<Utc>) {
        // Keep only last 24 hours
        self.questions.retain(|ts| now.signed_duration_since(*ts) < Duration::hours(24));
    }

    /// Check if user is in cooldown
    fn is_in_cooldown(&self, now: DateTime<Utc>) -> bool {
        if let Some(until) = self.cooldown_until {
            now < until
        } else {
            false
        }
    }

    /// Get remaining cooldown time in seconds
    fn remaining_cooldown(&self, now: DateTime<Utc>) -> Option<i64> {
        if let Some(until) = self.cooldown_until {
            if now < until {
                return Some(until.signed_duration_since(now).num_seconds());
            }
        }
        None
    }

    /// Count questions in time window
    fn count_in_window(&self, now: DateTime<Utc>, window: Duration) -> usize {
        self.questions
            .iter()
            .filter(|ts| now.signed_duration_since(**ts) < window)
            .count()
    }
}

/// Rate limiting result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub retry_after_seconds: Option<i64>,
}

/// Rate limiter for question submissions
pub struct RateLimiter {
    config: RateLimitConfig,
    users: Arc<Mutex<HashMap<String, UserState>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            config: RateLimitConfig::default(),
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            config,
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if user can ask a question
    pub fn check_rate_limit(&self, user_id: &str) -> RateLimitResult {
        let mut users = self.users.lock().unwrap();
        let now = Utc::now();

        let user_state = users.entry(user_id.to_string()).or_insert_with(UserState::new);

        // Cleanup old timestamps
        user_state.cleanup(now);

        // Check cooldown
        if user_state.is_in_cooldown(now) {
            return RateLimitResult {
                allowed: false,
                reason: Some(format!(
                    "Cooldown active. Please wait {} seconds.",
                    user_state.remaining_cooldown(now).unwrap_or(0)
                )),
                retry_after_seconds: user_state.remaining_cooldown(now),
            };
        }

        // Check per-minute limit
        let per_minute = user_state.count_in_window(now, Duration::minutes(1));
        if per_minute >= self.config.max_questions_per_minute {
            // Calculate seconds until next minute
            let retry_after = 60 - (now.timestamp() % 60);
            return RateLimitResult {
                allowed: false,
                reason: Some(format!(
                    "Rate limit exceeded: {} questions per minute. Try again in {} seconds.",
                    self.config.max_questions_per_minute, retry_after
                )),
                retry_after_seconds: Some(retry_after),
            };
        }

        // Check per-hour limit
        let per_hour = user_state.count_in_window(now, Duration::hours(1));
        if per_hour >= self.config.max_questions_per_hour {
            return RateLimitResult {
                allowed: false,
                reason: Some(format!(
                    "Rate limit exceeded: {} questions per hour.",
                    self.config.max_questions_per_hour
                )),
                retry_after_seconds: Some(3600 - now.signed_duration_since(user_state.questions[0]).num_seconds()),
            };
        }

        // Check per-day limit
        let per_day = user_state.count_in_window(now, Duration::hours(24));
        if per_day >= self.config.max_questions_per_day {
            return RateLimitResult {
                allowed: false,
                reason: Some(format!(
                    "Rate limit exceeded: {} questions per day.",
                    self.config.max_questions_per_day
                )),
                retry_after_seconds: Some(86400 - now.signed_duration_since(user_state.questions[0]).num_seconds()),
            };
        }

        RateLimitResult {
            allowed: true,
            reason: None,
            retry_after_seconds: None,
        }
    }

    /// Record a question attempt
    pub fn record_question(&self, user_id: &str) {
        let mut users = self.users.lock().unwrap();
        let now = Utc::now();

        let user_state = users.entry(user_id.to_string()).or_insert_with(UserState::new);
        user_state.questions.push(now);
    }

    /// Apply cooldown for violations
    pub fn apply_cooldown(&self, user_id: &str) {
        let mut users = self.users.lock().unwrap();
        let now = Utc::now();

        let user_state = users.entry(user_id.to_string()).or_insert_with(UserState::new);
        user_state.violations += 1;

        // Exponential backoff
        let cooldown_seconds = (self.config.initial_cooldown_seconds as f32
            * self.config.cooldown_multiplier.powi(user_state.violations as i32 - 1))
            as u64;
        let cooldown_seconds = cooldown_seconds.min(self.config.max_cooldown_seconds);

        user_state.cooldown_until = Some(now + Duration::seconds(cooldown_seconds as i64));
    }

    /// Reset user state (for testing or admin override)
    pub fn reset_user(&self, user_id: &str) {
        let mut users = self.users.lock().unwrap();
        users.remove(user_id);
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_questions_per_minute, 2);
        assert_eq!(config.max_questions_per_hour, 10);
        assert_eq!(config.max_questions_per_day, 50);
    }

    #[test]
    fn test_rate_limiter_allows_first_question() {
        let limiter = RateLimiter::new();
        let result = limiter.check_rate_limit("user1");
        assert!(result.allowed);
        assert!(result.reason.is_none());
    }

    #[test]
    fn test_rate_limiter_per_minute_limit() {
        let limiter = RateLimiter::new();

        // First 2 questions should be allowed
        assert!(limiter.check_rate_limit("user1").allowed);
        limiter.record_question("user1");
        
        assert!(limiter.check_rate_limit("user1").allowed);
        limiter.record_question("user1");

        // Third should be blocked
        let result = limiter.check_rate_limit("user1");
        assert!(!result.allowed);
        assert!(result.reason.is_some());
    }

    #[test]
    fn test_rate_limiter_different_users() {
        let limiter = RateLimiter::new();

        // User1 uses their quota
        assert!(limiter.check_rate_limit("user1").allowed);
        limiter.record_question("user1");
        assert!(limiter.check_rate_limit("user1").allowed);
        limiter.record_question("user1");

        // User2 should still be allowed
        assert!(limiter.check_rate_limit("user2").allowed);
    }

    #[test]
    fn test_cooldown_exponential_backoff() {
        let limiter = RateLimiter::new();

        limiter.apply_cooldown("user1");
        
        let users = limiter.users.lock().unwrap();
        let user_state = users.get("user1").unwrap();
        assert_eq!(user_state.violations, 1);
        assert!(user_state.cooldown_until.is_some());
    }

    #[test]
    fn test_reset_user() {
        let limiter = RateLimiter::new();

        limiter.record_question("user1");
        limiter.record_question("user1");
        assert!(!limiter.check_rate_limit("user1").allowed);

        limiter.reset_user("user1");
        assert!(limiter.check_rate_limit("user1").allowed);
    }

    #[test]
    fn test_user_state_cleanup() {
        let mut state = UserState::new();
        let now = Utc::now();

        // Add old timestamp (25 hours ago)
        state.questions.push(now - Duration::hours(25));
        // Add recent timestamp
        state.questions.push(now - Duration::minutes(5));

        state.cleanup(now);

        // Old timestamp should be removed
        assert_eq!(state.questions.len(), 1);
    }

    #[test]
    fn test_cooldown_check() {
        let mut state = UserState::new();
        let now = Utc::now();

        assert!(!state.is_in_cooldown(now));

        state.cooldown_until = Some(now + Duration::minutes(5));
        assert!(state.is_in_cooldown(now));

        let future = now + Duration::minutes(10);
        assert!(!state.is_in_cooldown(future));
    }
}

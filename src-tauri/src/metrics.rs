use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub last_request_time_ms: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time_ms: 0.0,
            last_request_time_ms: 0.0,
        }
    }
}

pub struct MetricsCollector {
    metrics: PerformanceMetrics,
    response_times: Vec<f64>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: PerformanceMetrics::default(),
            response_times: Vec::new(),
        }
    }

    pub fn start_request(&self) -> Instant {
        Instant::now()
    }

    pub fn record_success(&mut self, start_time: Instant) {
        let duration = start_time.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;

        self.metrics.total_requests += 1;
        self.metrics.successful_requests += 1;
        self.metrics.last_request_time_ms = duration_ms;

        self.response_times.push(duration_ms);
        if self.response_times.len() > 100 {
            self.response_times.remove(0);
        }

        self.update_average();
    }

    pub fn record_failure(&mut self, start_time: Instant) {
        let duration = start_time.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;

        self.metrics.total_requests += 1;
        self.metrics.failed_requests += 1;
        self.metrics.last_request_time_ms = duration_ms;
    }

    fn update_average(&mut self) {
        if !self.response_times.is_empty() {
            let sum: f64 = self.response_times.iter().sum();
            self.metrics.average_response_time_ms = sum / self.response_times.len() as f64;
        }
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.clone()
    }

    pub fn get_success_rate(&self) -> f64 {
        if self.metrics.total_requests == 0 {
            return 0.0;
        }
        (self.metrics.successful_requests as f64 / self.metrics.total_requests as f64) * 100.0
    }
}

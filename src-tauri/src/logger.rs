use chrono::{DateTime, Local};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Success,
    Network,
    Config,
    Metrics,
}

impl LogLevel {
    pub fn emoji(&self) -> &str {
        match self {
            LogLevel::Debug => "🐛",
            LogLevel::Info => "ℹ️",
            LogLevel::Warning => "⚠️",
            LogLevel::Error => "❌",
            LogLevel::Success => "✅",
            LogLevel::Network => "📡",
            LogLevel::Config => "🔧",
            LogLevel::Metrics => "📊",
        }
    }

    pub fn color_code(&self) -> &str {
        match self {
            LogLevel::Debug => "\x1b[36m",      // Cyan
            LogLevel::Info => "\x1b[34m",       // Blue
            LogLevel::Warning => "\x1b[33m",    // Yellow
            LogLevel::Error => "\x1b[31m",      // Red
            LogLevel::Success => "\x1b[32m",    // Green
            LogLevel::Network => "\x1b[35m",    // Magenta
            LogLevel::Config => "\x1b[95m",     // Bright Magenta
            LogLevel::Metrics => "\x1b[96m",    // Bright Cyan
        }
    }
}

pub struct Logger {
    debug_enabled: AtomicBool,
}

impl Logger {
    pub fn new(debug_enabled: bool) -> Self {
        Self { 
            debug_enabled: AtomicBool::new(debug_enabled)
        }
    }

    pub fn set_debug_enabled(&self, enabled: bool) {
        self.debug_enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn log(&self, level: LogLevel, component: &str, message: &str) {
        // Skip debug logs if debug is disabled
        if matches!(level, LogLevel::Debug) && !self.debug_enabled.load(Ordering::Relaxed) {
            return;
        }

        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%H:%M:%S%.3f");
        let reset = "\x1b[0m";

        println!(
            "{}{} [{}] {} {} {}{}",
            level.color_code(),
            level.emoji(),
            timestamp,
            component,
            reset,
            message,
            reset
        );
    }

    pub fn debug(&self, component: &str, message: &str) {
        self.log(LogLevel::Debug, component, message);
    }

    pub fn info(&self, component: &str, message: &str) {
        self.log(LogLevel::Info, component, message);
    }

    pub fn warn(&self, component: &str, message: &str) {
        self.log(LogLevel::Warning, component, message);
    }

    pub fn error(&self, component: &str, message: &str) {
        self.log(LogLevel::Error, component, message);
    }

    pub fn success(&self, component: &str, message: &str) {
        self.log(LogLevel::Success, component, message);
    }

    pub fn network(&self, component: &str, message: &str) {
        self.log(LogLevel::Network, component, message);
    }

    pub fn config(&self, component: &str, message: &str) {
        self.log(LogLevel::Config, component, message);
    }

    pub fn metrics(&self, component: &str, message: &str) {
        self.log(LogLevel::Metrics, component, message);
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

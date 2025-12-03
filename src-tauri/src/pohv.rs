use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_TIMEOUT_SECS: u64 = 3600; // 1 hour
const WARNING_THRESHOLD_SECS: u64 = 300; // 5 minutes warning

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PoHVStatus {
    Active,
    Warning,
    Locked,
}

#[derive(Clone, Serialize)]
pub struct PoHVState {
    pub status: PoHVStatus,
    pub seconds_remaining: u64,
    pub last_interaction: u64,
}

pub struct PoHVSystem {
    last_interaction: Mutex<SystemTime>,
    timeout_duration: Duration,
    warning_threshold: Duration,
    status: Mutex<PoHVStatus>,
}

impl PoHVSystem {
    pub fn new() -> Self {
        Self {
            last_interaction: Mutex::new(SystemTime::now()),
            timeout_duration: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            warning_threshold: Duration::from_secs(WARNING_THRESHOLD_SECS),
            status: Mutex::new(PoHVStatus::Active),
        }
    }

    pub fn register_heartbeat(&self) {
        let mut last = self.last_interaction.lock().unwrap();
        *last = SystemTime::now();
        
        let mut status = self.status.lock().unwrap();
        *status = PoHVStatus::Active;
    }

    pub fn get_state(&self) -> PoHVState {
        let last = *self.last_interaction.lock().unwrap();
        let now = SystemTime::now();
        
        let elapsed = now.duration_since(last).unwrap_or(Duration::from_secs(0));
        let remaining = if elapsed < self.timeout_duration {
            self.timeout_duration - elapsed
        } else {
            Duration::from_secs(0)
        };

        let current_status = if remaining.as_secs() == 0 {
            PoHVStatus::Locked
        } else if remaining < self.warning_threshold {
            PoHVStatus::Warning
        } else {
            PoHVStatus::Active
        };

        // Update internal status cache
        {
            let mut status_guard = self.status.lock().unwrap();
            *status_guard = current_status.clone();
        }

        PoHVState {
            status: current_status,
            seconds_remaining: remaining.as_secs(),
            last_interaction: last.duration_since(UNIX_EPOCH).unwrap().as_secs(),
        }
    }

    pub fn is_locked(&self) -> bool {
        self.get_state().status == PoHVStatus::Locked
    }
}

// Tauri Commands

#[tauri::command]
pub fn pohv_heartbeat(state: tauri::State<crate::state::AppState>) -> PoHVState {
    state.pohv_system.register_heartbeat();
    state.pohv_system.get_state()
}

#[tauri::command]
pub fn pohv_get_status(state: tauri::State<crate::state::AppState>) -> PoHVState {
    state.pohv_system.get_state()
}

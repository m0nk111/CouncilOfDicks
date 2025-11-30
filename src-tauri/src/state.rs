use std::sync::{Arc, Mutex};
use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
        }
    }

    pub fn get_config(&self) -> AppConfig {
        self.config.lock().unwrap().clone()
    }

    pub fn update_config<F>(&self, f: F)
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = self.config.lock().unwrap();
        f(&mut config);
    }
}

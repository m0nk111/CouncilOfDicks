use std::sync::{Arc, Mutex};
use crate::config::AppConfig;
use crate::council::CouncilSessionManager;
use crate::logger::Logger;
use crate::mcp::McpServer;
use crate::metrics::MetricsCollector;
use crate::p2p_manager::P2PManager;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub logger: Arc<Logger>,
    pub metrics: Arc<Mutex<MetricsCollector>>,
    pub p2p_manager: Arc<P2PManager>,
    pub council_manager: Arc<CouncilSessionManager>,
    pub mcp_server: Arc<McpServer>,
}

impl AppState {
    pub fn new() -> Self {
        let logger = Arc::new(Logger::new(false)); // Debug disabled by default
        let council_manager = Arc::new(CouncilSessionManager::new());
        let mcp_server = Arc::new(McpServer::new(9001, council_manager.clone(), logger.clone()));
        
        Self {
            config: Arc::new(Mutex::new(AppConfig::new())),
            logger: logger.clone(),
            metrics: Arc::new(Mutex::new(MetricsCollector::new())),
            p2p_manager: Arc::new(P2PManager::new(9000)),
            council_manager,
            mcp_server,
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

    pub fn log_debug(&self, component: &str, message: &str) {
        self.logger.debug(component, message);
    }

    pub fn log_info(&self, component: &str, message: &str) {
        self.logger.info(component, message);
    }

    pub fn log_error(&self, component: &str, message: &str) {
        self.logger.error(component, message);
    }

    pub fn log_success(&self, component: &str, message: &str) {
        self.logger.success(component, message);
    }

    pub fn log_network(&self, component: &str, message: &str) {
        self.logger.network(component, message);
    }
}

use std::sync::{Arc, Mutex};
use crate::chat::ChannelManager;
use crate::config::AppConfig;
use crate::council::CouncilSessionManager;
use crate::crypto::SigningIdentity;
use crate::knowledge::KnowledgeBank;
use crate::logger::Logger;
use crate::mcp::McpServer;
use crate::metrics::MetricsCollector;
use crate::p2p_manager::P2PManager;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub logger: Arc<Logger>,
    pub metrics: Arc<Mutex<MetricsCollector>>,
    pub p2p_manager: Arc<P2PManager>,
    pub council_manager: Arc<CouncilSessionManager>,
    pub mcp_server: Arc<McpServer>,
    pub signing_identity: Arc<SigningIdentity>,
    pub knowledge_bank: Option<Arc<KnowledgeBank>>,
    pub channel_manager: Arc<ChannelManager>,
}

impl AppState {
    pub fn new() -> Self {
        let logger = Arc::new(Logger::new(false)); // Debug disabled by default
        let council_manager = Arc::new(CouncilSessionManager::new());
        let mcp_server = Arc::new(McpServer::new(9001, council_manager.clone(), logger.clone()));
        
        // Load or generate signing identity
        let keypair_path = PathBuf::from("./council_identity.key");
        let signing_identity = if keypair_path.exists() {
            logger.info("crypto", "Loading existing signing identity");
            match SigningIdentity::load(keypair_path.clone()) {
                Ok(identity) => {
                    logger.success("crypto", &format!("Identity loaded: {}", 
                        identity.public_key_base64()[..16].to_string()));
                    Arc::new(identity)
                },
                Err(e) => {
                    logger.error("crypto", &format!("Failed to load identity: {}", e));
                    logger.info("crypto", "Generating new identity");
                    let identity = SigningIdentity::generate();
                    let _ = identity.save(keypair_path);
                    Arc::new(identity)
                }
            }
        } else {
            logger.info("crypto", "Generating new signing identity");
            let identity = SigningIdentity::generate();
            match identity.save(keypair_path) {
                Ok(_) => logger.success("crypto", "Identity saved to council_identity.key"),
                Err(e) => logger.error("crypto", &format!("Failed to save identity: {}", e)),
            }
            Arc::new(identity)
        };
        
        // Initialize knowledge bank (async operation, will be None if fails)
        let kb = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                let config = AppConfig::default();
                match KnowledgeBank::new(
                    "sqlite:./council_knowledge.db",
                    logger.clone(),
                    config.ollama_url.clone(),
                )
                .await
                {
                    Ok(kb) => Some(Arc::new(kb)),
                    Err(e) => {
                        logger.error("knowledge", &format!("Failed to initialize Knowledge Bank: {}", e));
                        None
                    }
                }
            });

        // Initialize channel manager
        let channel_manager = Arc::new(ChannelManager::new());
        
        // Send welcome message to #general
        let _ = channel_manager.send_system_message(
            crate::chat::ChannelType::General,
            "🤖 Welcome to Council Of Dicks! Type /help for commands.".to_string(),
        );

        Self {
            config: Arc::new(Mutex::new(AppConfig::default())),
            logger: logger.clone(),
            metrics: Arc::new(Mutex::new(MetricsCollector::new())),
            p2p_manager: Arc::new(P2PManager::new(9000)),
            council_manager,
            mcp_server,
            signing_identity,
            knowledge_bank: kb,
            channel_manager,
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

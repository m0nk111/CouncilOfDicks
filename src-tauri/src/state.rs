use crate::agents::{Agent, AgentPool};
use crate::chat::{
    ChannelManager, DuplicateFilter, Message as ChatMessage, RateLimiter, SpamDetector,
};
use crate::chat_bot::{ChatBot, ChatBotStatus};
use crate::config::AppConfig;
use crate::council::CouncilSessionManager;
use crate::crypto::SigningIdentity;
use crate::knowledge::KnowledgeBank;
use crate::logger::Logger;
use crate::mcp::McpServer;
use crate::metrics::MetricsCollector;
use crate::p2p_manager::P2PManager;
use crate::pohv::PoHVSystem;
use crate::reputation::ReputationManager;
use crate::topic_manager::TopicManager;
use crate::constitution::ConstitutionManager;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

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
    pub duplicate_filter: Option<Arc<DuplicateFilter>>,
    pub rate_limiter: Arc<RateLimiter>,
    pub spam_detector: Arc<SpamDetector>,
    pub websocket_broadcast: Arc<broadcast::Sender<ChatMessage>>,
    pub agent_pool: Arc<AgentPool>,
    pub pohv_system: Arc<PoHVSystem>,
    pub topic_manager: Arc<TopicManager>,
    pub reputation_manager: Arc<ReputationManager>,
    pub chat_bot_status: Arc<Mutex<ChatBotStatus>>,
    pub constitution_manager: Arc<ConstitutionManager>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        tokio::runtime::Runtime::new()
            .expect("tokio runtime for AppState")
            .block_on(Self::initialize())
    }

    pub async fn initialize() -> Self {
        let mut base_config = AppConfig::load();
        
        // Load API keys from ~/.secrets/keys/ if not in config
        base_config.load_api_keys_from_files();
        
        let logger = Arc::new(Logger::new(false));
        logger.set_debug_enabled(base_config.debug_enabled);
        logger.info("config", &format!("Loaded configuration (handle: {})", base_config.user_handle));
        
        // Log available providers
        let providers = base_config.available_providers();
        logger.info("providers", &format!("Available providers: {:?}", providers));

        // Ensure data directory exists for persistence
        let data_dir = PathBuf::from("./data");
        if let Err(e) = fs::create_dir_all(&data_dir) {
            logger.warn("storage", &format!("⚠️ Could not prepare data dir: {}", e));
        }

        // Initialize knowledge bank
        let kb_path = data_dir.join("knowledge_bank.sqlite");
        let kb_url = format!("sqlite://{}", kb_path.to_string_lossy());
        
        // Build auth tuple - Ollama Guardian uses username-only (app name), password optional
        let ollama_auth = base_config.ollama_username.as_ref().map(|u| {
            (u.clone(), base_config.ollama_password.clone().unwrap_or_default())
        });
        
        let knowledge_bank =
            match KnowledgeBank::new(&kb_url, logger.clone(), base_config.ollama_url.clone(), ollama_auth).await
            {
                Ok(bank) => Some(Arc::new(bank)),
                Err(e) => {
                    logger.warn("knowledge", &format!("⚠️ Knowledge bank disabled: {}", e));
                    None
                }
            };

        let council_manager = Arc::new(CouncilSessionManager::new(knowledge_bank.clone()));
        
        // Load sessions from DB
        council_manager.load_from_db().await;

        let channel_manager = Arc::new(ChannelManager::new(knowledge_bank.clone()));
        // Load chat history
        channel_manager.load_history().await;

        let mcp_server = Arc::new(McpServer::new(
            9001,
            council_manager.clone(),
            logger.clone(),
        ));

        let chat_bot_status = Arc::new(Mutex::new(ChatBotStatus::default()));

        // Load or generate signing identity (stored in data/ directory)
        let keypair_path = data_dir.join("council_identity.key");
        let signing_identity = if keypair_path.exists() {
            logger.info("crypto", "Loading existing signing identity");
            match SigningIdentity::load(keypair_path.clone()) {
                Ok(identity) => {
                    logger.success(
                        "crypto",
                        &format!(
                            "Identity loaded: {}",
                            &identity.public_key_base64()[..16]
                        ),
                    );
                    Arc::new(identity)
                }
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
            match identity.save(keypair_path.clone()) {
                Ok(_) => logger.success("crypto", "Identity saved to council_identity.key"),
                Err(e) => logger.error("crypto", &format!("Failed to save identity: {}", e)),
            }
            Arc::new(identity)
        };

        // Initialize channel manager
        let channel_manager = Arc::new(ChannelManager::new(knowledge_bank.clone()));
        let _ = channel_manager.send_system_message(
            crate::chat::ChannelType::General,
            "🤖 Welcome to Council Of Dicks! Type /help for commands.".to_string(),
        );

        // Initialize duplicate filter if KB is available
        let duplicate_filter = knowledge_bank
            .as_ref()
            .map(|kb| Arc::new(DuplicateFilter::new(Arc::clone(kb))));

        let rate_limiter = Arc::new(RateLimiter::new());
        let spam_detector = Arc::new(SpamDetector::new());
        let (ws_tx, _ws_rx) = broadcast::channel::<ChatMessage>(100);
        let agent_pool = Arc::new(AgentPool::new());

        // Load agents from config/agents.json
        let mut agents_config_path = std::path::PathBuf::from("config/agents.json");
        if !agents_config_path.exists() {
            // Try parent directory (for when running from src-tauri)
            let parent_path = std::path::PathBuf::from("../config/agents.json");
            if parent_path.exists() {
                agents_config_path = parent_path;
            }
        }

        if agents_config_path.exists() {
            logger.info("agent", &format!("Loading agents from {:?}", agents_config_path));
            
            // Set config path for persistence
            agent_pool.set_config_path(agents_config_path.clone()).await;
            
            if let Ok(content) = fs::read_to_string(&agents_config_path) {
                #[derive(serde::Deserialize)]
                struct AgentConfig {
                    name: String,
                    handle: Option<String>,
                    provider: Option<String>,
                    model: String,
                    system_prompt: String,
                    timeout_secs: Option<u64>,
                    metadata: Option<std::collections::HashMap<String, String>>,
                }

                match serde_json::from_str::<Vec<AgentConfig>>(&content) {
                    Ok(configs) => {
                        for config in configs {
                            let provider = config.provider.unwrap_or_else(|| "ollama".to_string());
                            let mut agent = Agent::with_provider(
                                config.name.clone(),
                                provider,
                                config.model.clone(),
                                config.system_prompt,
                            );
                            if let Some(handle) = config.handle {
                                agent.handle = handle;
                            }
                            if let Some(timeout) = config.timeout_secs {
                                agent.timeout_secs = Some(timeout);
                            }
                            if let Some(metadata) = config.metadata {
                                agent.metadata = metadata;
                            }
                            if let Err(e) = agent_pool.add_agent(agent).await {
                                logger.error("agent", &format!("Failed to add agent {}: {}", config.name, e));
                            } else {
                                let timeout_info = config.timeout_secs.map(|t| format!(" (timeout: {}s)", t)).unwrap_or_default();
                                logger.success("agent", &format!("Loaded agent: {} ({}){}", config.name, config.model, timeout_info));
                            }
                        }
                    }
                    Err(e) => {
                        logger.error("agent", &format!("Failed to parse agents.json: {}", e));
                    }
                }
            } else {
                logger.error("agent", &format!("Failed to read {:?}", agents_config_path));
            }
        } else {
            logger.warn("agent", "config/agents.json not found (checked ./config/agents.json and ../config/agents.json). No agents loaded.");
        }
        let pohv_system = Arc::new(PoHVSystem::new());
        let topic_manager = Arc::new(TopicManager::new());
        let reputation_manager = Arc::new(ReputationManager::new(knowledge_bank.clone()));
        let constitution_manager = Arc::new(ConstitutionManager::new());
        
        // Load reputations from DB
        reputation_manager.load_from_db().await;

        // Initialize topic if configured
        if let Some(topic) = &base_config.initial_topic {
            topic_manager.force_set_topic(topic.clone(), Some(base_config.topic_interval));
        }

        let p2p_port = base_config.p2p_port;
        let bootstrap_peers = base_config.bootstrap_peers.clone();

        let state = Self {
            config: Arc::new(Mutex::new(base_config)),
            logger: logger.clone(),
            metrics: Arc::new(Mutex::new(MetricsCollector::new())),
            p2p_manager: Arc::new(P2PManager::new(p2p_port, bootstrap_peers)),
            council_manager,
            mcp_server,
            signing_identity,
            knowledge_bank,
            channel_manager,
            duplicate_filter,
            rate_limiter,
            spam_detector,
            websocket_broadcast: Arc::new(ws_tx),
            agent_pool: agent_pool.clone(),
            pohv_system,
            topic_manager,
            reputation_manager,
            chat_bot_status,
            constitution_manager,
        };
        
        // Start background tasks
        crate::topic_manager::start_topic_loop(Arc::new(state.clone()));

        // Start ChatBot monitoring
        let chat_bot_state = Arc::new(state.clone());
        let chat_bot_agents = agent_pool.clone();
        tokio::spawn(async move {
            let mut chat_bot = ChatBot::new(chat_bot_state, chat_bot_agents);
            chat_bot.start_monitoring().await;
        });

        // Start P2P event loop
        let p2p_state = Arc::new(state.clone());
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                p2p_state.p2p_manager.process_events(p2p_state.clone()).await;
            }
        });

        state
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
        if let Err(e) = config.save() {
            self.logger.error("config", &format!("Failed to save config: {}", e));
        }
    }

    pub fn log_debug(&self, component: &str, message: &str) {
        self.logger.debug(component, message);
    }

    pub fn log_info(&self, component: &str, message: &str) {
        self.logger.info(component, message);
    }

    pub fn log_warn(&self, component: &str, message: &str) {
        self.logger.warn(component, message);
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

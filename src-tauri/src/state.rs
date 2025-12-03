use crate::agents::AgentPool;
use crate::chat::{
    ChannelManager, DuplicateFilter, Message as ChatMessage, RateLimiter, SpamDetector,
};
use crate::config::AppConfig;
use crate::council::CouncilSessionManager;
use crate::crypto::SigningIdentity;
use crate::knowledge::KnowledgeBank;
use crate::logger::Logger;
use crate::mcp::McpServer;
use crate::metrics::MetricsCollector;
use crate::p2p_manager::P2PManager;
use crate::pohv::PoHVSystem;
use crate::topic_manager::TopicManager;
use crate::verdict_store::VerdictStore;
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
    pub verdict_store: Option<Arc<VerdictStore>>,
    pub pohv_system: Arc<PoHVSystem>,
    pub topic_manager: Arc<TopicManager>,
}

impl AppState {
    pub fn new() -> Self {
        tokio::runtime::Runtime::new()
            .expect("tokio runtime for AppState")
            .block_on(Self::initialize())
    }

    pub async fn initialize() -> Self {
        let base_config = AppConfig::default();
        let logger = Arc::new(Logger::new(false));
        logger.set_debug_enabled(base_config.debug_enabled);

        let council_manager = Arc::new(CouncilSessionManager::new());
        let mcp_server = Arc::new(McpServer::new(
            9001,
            council_manager.clone(),
            logger.clone(),
        ));

        // Load or generate signing identity
        let keypair_path = PathBuf::from("./council_identity.key");
        let signing_identity = if keypair_path.exists() {
            logger.info("crypto", "Loading existing signing identity");
            match SigningIdentity::load(keypair_path.clone()) {
                Ok(identity) => {
                    logger.success(
                        "crypto",
                        &format!(
                            "Identity loaded: {}",
                            identity.public_key_base64()[..16].to_string()
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

        // Ensure data directory exists for persistence
        let data_dir = PathBuf::from("./data");
        if let Err(e) = fs::create_dir_all(&data_dir) {
            logger.warn("storage", &format!("⚠️ Could not prepare data dir: {}", e));
        }

        // Initialize knowledge bank
        let kb_path = data_dir.join("knowledge_bank.sqlite");
        let kb_url = format!("sqlite://{}", kb_path.to_string_lossy());
        let knowledge_bank =
            match KnowledgeBank::new(&kb_url, logger.clone(), base_config.ollama_url.clone()).await
            {
                Ok(bank) => Some(Arc::new(bank)),
                Err(e) => {
                    logger.warn("knowledge", &format!("⚠️ Knowledge bank disabled: {}", e));
                    None
                }
            };

        // Initialize verdict store
        let verdicts_path = data_dir.join("council_verdicts.sqlite");
        let verdicts_url = format!("sqlite://{}", verdicts_path.to_string_lossy());
        let verdict_store = match VerdictStore::new(&verdicts_url, logger.clone()).await {
            Ok(store) => Some(Arc::new(store)),
            Err(e) => {
                logger.error(
                    "verdict_store",
                    &format!("❌ Verdict storage disabled: {}", e),
                );
                None
            }
        };

        // Initialize channel manager
        let channel_manager = Arc::new(ChannelManager::new());
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
        let pohv_system = Arc::new(PoHVSystem::new());
        let topic_manager = Arc::new(TopicManager::new());

        // Start the topic loop
        // We need to do this after we have the full AppState, which is tricky in `initialize`
        // because we are constructing it.
        // We'll have to start it separately or use a lazy initialization pattern.
        // For now, let's just create it here.

        // Initialize topic if configured
        if let Some(topic) = &base_config.initial_topic {
            topic_manager.set_topic(topic.clone(), Some(base_config.topic_interval));
        }

        let state = Self {
            config: Arc::new(Mutex::new(base_config)),
            logger: logger.clone(),
            metrics: Arc::new(Mutex::new(MetricsCollector::new())),
            p2p_manager: Arc::new(P2PManager::new(9000)),
            council_manager,
            mcp_server,
            signing_identity,
            knowledge_bank,
            channel_manager,
            duplicate_filter,
            rate_limiter,
            spam_detector,
            websocket_broadcast: Arc::new(ws_tx),
            agent_pool,
            verdict_store: verdict_store,
            pohv_system,
            topic_manager,
        };
        
        // Start background tasks
        crate::topic_manager::start_topic_loop(Arc::new(state.clone()));

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

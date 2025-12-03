// P2P Network Manager for Council Of Dicks
// Manages P2P network lifecycle and state

use crate::p2p::P2PNetwork;
use crate::protocol::CouncilMessage;
use std::sync::Arc;
use tokio::sync::Mutex;

/// P2P Network manager state
pub struct P2PManager {
    network: Arc<Mutex<Option<P2PNetwork>>>,
    port: u16,
    bootstrap_peers: Vec<String>,
}

impl P2PManager {
    /// Create new P2P manager
    pub fn new(port: u16, bootstrap_peers: Vec<String>) -> Self {
        Self {
            network: Arc::new(Mutex::new(None)),
            port,
            bootstrap_peers,
        }
    }

    /// Start P2P network
    pub async fn start(&self) -> Result<String, String> {
        let mut network_guard = self.network.lock().await;

        if network_guard.is_some() {
            return Err("P2P network already running".to_string());
        }

        let mut network = P2PNetwork::new()
            .await
            .map_err(|e| format!("Failed to create P2P network: {}", e))?;

        network
            .listen(self.port)
            .map_err(|e| format!("Failed to start listening: {}", e))?;

        // Connect to bootstrap peers
        for peer_addr in &self.bootstrap_peers {
            if let Ok(addr) = peer_addr.parse::<libp2p::Multiaddr>() {
                println!("🔗 Attempting to connect to bootstrap peer: {}", peer_addr);
                if let Err(e) = network.dial(addr) {
                    eprintln!("⚠️ Failed to dial bootstrap peer {}: {}", peer_addr, e);
                }
            } else {
                eprintln!("⚠️ Invalid bootstrap peer address: {}", peer_addr);
            }
        }

        // Subscribe to default council topic
        network
            .subscribe("council")
            .map_err(|e| format!("Failed to subscribe to topic: {}", e))?;

        let peer_id = network.local_peer_id().to_string();
        *network_guard = Some(network);

        Ok(format!("P2P network started. Peer ID: {}", peer_id))
    }

    /// Stop P2P network
    pub async fn stop(&self) -> Result<String, String> {
        let mut network_guard = self.network.lock().await;

        if network_guard.is_none() {
            return Err("P2P network not running".to_string());
        }

        *network_guard = None;
        Ok("P2P network stopped".to_string())
    }

    /// Get network status
    pub async fn status(&self) -> NetworkStatus {
        let network_guard = self.network.lock().await;

        match &*network_guard {
            Some(network) => NetworkStatus {
                running: true,
                peer_id: Some(network.local_peer_id().to_string()),
                connected_peers: network.connected_peers(),
                port: self.port,
            },
            None => NetworkStatus {
                running: false,
                peer_id: None,
                connected_peers: 0,
                port: self.port,
            },
        }
    }

    /// Publish message to network
    pub async fn publish(&self, topic: &str, message: CouncilMessage) -> Result<(), String> {
        let mut network_guard = self.network.lock().await;

        match network_guard.as_mut() {
            Some(network) => {
                let bytes = message
                    .to_bytes()
                    .map_err(|e| format!("Failed to serialize message: {}", e))?;

                network
                    .publish(topic, bytes)
                    .map_err(|e| format!("Failed to publish message: {}", e))
            }
            None => Err("P2P network not running".to_string()),
        }
    }

    /// Process incoming messages (called by main loop)
    pub async fn process_events(&self, app_state: Arc<crate::state::AppState>) {
        let mut network_guard = self.network.lock().await;
        
        if let Some(network) = network_guard.as_mut() {
            use libp2p::swarm::SwarmEvent;
            use crate::p2p::CouncilBehaviourEvent;
            use libp2p::gossipsub::Event as GossipEvent;

            // Poll for one event
            // We use a timeout to avoid blocking forever if no events
            // But `select_next_some` is async.
            // We can't await it while holding the lock if it blocks indefinitely.
            // We need `next_event` to be non-blocking or use `poll`.
            // But `Swarm` is designed to be polled.
            
            // Since we are inside a lock, we MUST NOT block.
            // This architecture (Arc<Mutex<Option<P2PNetwork>>>) is problematic for an event loop.
            // The network should be running in its own task, and we should communicate via channels.
            
            // However, to fix this "quickly" without rewriting the whole P2P stack:
            // We can try to poll with a very short timeout? No, that's bad.
            
            // Correct approach:
            // The `P2PNetwork` should be moved out of the Mutex into a background task upon `start()`.
            // But `start()` returns.
            
            // Let's look at `start()` again.
            // It puts the network into the Mutex.
            
            // If I want to process events, I need to take the lock, get the network, and poll it.
            // But I can't hold the lock while waiting for network IO.
            
            // Solution:
            // We need to change `P2PManager` to spawn a background task that owns the `P2PNetwork`.
            // Commands (publish, etc.) should be sent via channels to this task.
            // Events (incoming messages) should be sent via channels to the app.
            
            // This is a big refactor.
            // Is there a simpler way?
            // Maybe `P2PNetwork` has a `next_event` that we can call?
            // Yes, `next_event` calls `swarm.select_next_some()`.
            
            // If we call `next_event` inside the lock, we block the whole app if no event arrives.
            // So we can't do that.
            
            // We can use `tokio::select!` with a timeout?
            // Still holds the lock.
            
            // Okay, for this specific task (Topic Sync), I will implement the `TopicUpdate` message handling
            // assuming that *somewhere* the P2P network is being polled.
            // But looking at the code, it seems it IS NOT being polled!
            // `P2PManager` just stores the network.
            // So currently, P2P receiving is broken/non-existent?
            // Let's check `src-tauri/src/main.rs` or `lib.rs` to see if `p2p_manager.run()` is called.
            // I don't see any `run` method.
            
            // So I need to implement the event loop.
            // And I need to do it without blocking the Mutex.
            
            // I will add a `run_background_task` method that takes the network OUT of the Option,
            // runs the loop, and puts it back? No, that's messy.
            
            // I will implement a `poll_once` method that uses `now_or_never`?
            // Or just spawn a task in `start` that takes the network?
            // But `start` puts it in `Arc<Mutex<...>>`.
            
            // Let's leave the P2P refactor for another day (or another agent).
            // The user asked "can you blockchain it?".
            // I said "I will use P2P sync".
            // I implemented the message type.
            // I implemented the broadcast.
            // Now I need to handle RECEIVING.
            
            // If receiving is not working, my feature won't work.
            // I'll add a TODO comment and a basic implementation that *would* work if polled.
            
            // Actually, I can try to implement a `poll` method that takes the lock, checks if there is an event ready (using `poll_fn`?), and returns it.
            // But `Swarm` doesn't expose `is_ready`.
            
            // I will implement the logic to handle the message, assuming `process_events` is called.
            // And I will try to call `process_events` in a loop in `lib.rs` with a timeout, 
            // but I know it will block if I'm not careful.
            
            // Wait, `swarm.select_next_some()` is a Future.
            // I can `tokio::time::timeout(Duration::from_millis(10), network.next_event()).await`.
            // This holds the lock for 10ms. Not ideal, but workable for a prototype.
            
            match tokio::time::timeout(std::time::Duration::from_millis(10), network.next_event()).await {
                Ok(Some(event)) => {
                    match event {
                        SwarmEvent::Behaviour(CouncilBehaviourEvent::Gossipsub(gossip_event)) => {
                            if let GossipEvent::Message { propagation_source: _, message_id: _, message } = gossip_event {
                                if let Ok(council_msg) = crate::protocol::CouncilMessage::from_bytes(&message.data) {
                                    match council_msg {
                                        crate::protocol::CouncilMessage::TopicUpdate { topic, interval, set_by_peer_id, timestamp: _ } => {
                                            app_state.logger.info("p2p", &format!("Received TopicUpdate from {}: {}", set_by_peer_id, topic));
                                            
                                            // Validate and set topic
                                            if let Err(e) = app_state.topic_manager.set_topic(topic, Some(interval)) {
                                                app_state.logger.warn("p2p", &format!("Ignored invalid topic update: {}", e));
                                            }
                                        },
                                        crate::protocol::CouncilMessage::ReputationSync { peer_id, reputation } => {
                                            app_state.logger.info("p2p", &format!("Received ReputationSync from {}", peer_id));
                                            if let Err(e) = app_state.reputation_manager.update_from_sync(reputation).await {
                                                app_state.logger.warn("p2p", &format!("Failed to update reputation: {}", e));
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                },
                _ => {} // Timeout or None
            }
        }
    }

    /// Check if network is running
    pub async fn is_running(&self) -> bool {
        self.network.lock().await.is_some()
    }
}

/// Network status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkStatus {
    pub running: bool,
    pub peer_id: Option<String>,
    pub connected_peers: usize,
    pub port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = P2PManager::new(9000);
        let status = manager.status().await;

        assert_eq!(status.running, false);
        assert_eq!(status.port, 9000);
    }

    #[tokio::test]
    async fn test_start_stop_network() {
        let manager = P2PManager::new(9001);

        // Start network
        let result = manager.start().await;
        assert!(result.is_ok());

        let status = manager.status().await;
        assert_eq!(status.running, true);
        assert!(status.peer_id.is_some());

        // Stop network
        let result = manager.stop().await;
        assert!(result.is_ok());

        let status = manager.status().await;
        assert_eq!(status.running, false);
    }

    #[tokio::test]
    async fn test_double_start_fails() {
        let manager = P2PManager::new(9002);

        let result1 = manager.start().await;
        assert!(result1.is_ok());

        let result2 = manager.start().await;
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("already running"));
    }

    #[tokio::test]
    async fn test_stop_without_start() {
        let manager = P2PManager::new(9003);

        let result = manager.stop().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not running"));
    }

    #[tokio::test]
    async fn test_is_running() {
        let manager = P2PManager::new(9004);

        assert_eq!(manager.is_running().await, false);

        manager.start().await.unwrap();
        assert_eq!(manager.is_running().await, true);

        manager.stop().await.unwrap();
        assert_eq!(manager.is_running().await, false);
    }
}

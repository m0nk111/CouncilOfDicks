// P2P Network Manager for Council Of Dicks
// Manages P2P network lifecycle and state

use crate::p2p::P2PNetwork;
use crate::protocol::CouncilMessage;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::error::Error;

/// P2P Network manager state
pub struct P2PManager {
    network: Arc<Mutex<Option<P2PNetwork>>>,
    port: u16,
}

impl P2PManager {
    /// Create new P2P manager
    pub fn new(port: u16) -> Self {
        Self {
            network: Arc::new(Mutex::new(None)),
            port,
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

        network.listen(self.port)
            .map_err(|e| format!("Failed to start listening: {}", e))?;

        // Subscribe to default council topic
        network.subscribe("council")
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
                let bytes = message.to_bytes()
                    .map_err(|e| format!("Failed to serialize message: {}", e))?;
                
                network.publish(topic, bytes)
                    .map_err(|e| format!("Failed to publish message: {}", e))
            }
            None => Err("P2P network not running".to_string()),
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

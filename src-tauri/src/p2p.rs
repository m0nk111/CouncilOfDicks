// P2P networking module for Council Of Dicks
// Decentralized mesh network using libp2p

use libp2p::{
    futures::StreamExt,
    gossipsub, identify, kad, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, Transport,
};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;

/// Network behavior combining all protocols
#[derive(NetworkBehaviour)]
pub struct CouncilBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
}

/// P2P network manager
pub struct P2PNetwork {
    swarm: Swarm<CouncilBehaviour>,
    local_peer_id: PeerId,
}

impl P2PNetwork {
    /// Create new P2P network instance
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        // Generate keypair for this node
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        println!("🔑 Local peer id: {}", local_peer_id);

        // Create transport with noise encryption and yamux multiplexing
        let transport = tcp::tokio::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        // Configure Gossipsub for message broadcasting
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(|message: &gossipsub::Message| {
                let mut hasher = DefaultHasher::new();
                message.data.hash(&mut hasher);
                gossipsub::MessageId::from(hasher.finish().to_string())
            })
            .build()
            .map_err(|e| format!("Gossipsub config error: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )?;

        // Configure mDNS for local peer discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        // Configure Identify protocol
        let identify = identify::Behaviour::new(
            identify::Config::new("/council/1.0.0".to_string(), local_key.public())
                .with_agent_version("CouncilOfDicks/0.1.0".to_string()),
        );

        // Configure Kademlia DHT
        let store = kad::store::MemoryStore::new(local_peer_id);
        let kademlia = kad::Behaviour::new(local_peer_id, store);

        // Combine all behaviors
        let behaviour = CouncilBehaviour {
            gossipsub,
            mdns,
            identify,
            kademlia,
        };

        // Create swarm
        let swarm = Swarm::new(
            transport,
            behaviour,
            local_peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );

        Ok(Self {
            swarm,
            local_peer_id,
        })
    }

    /// Start listening on all interfaces
    pub fn listen(&mut self, port: u16) -> Result<(), Box<dyn Error>> {
        let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", port)
            .parse()
            .map_err(|e| format!("Invalid listen address: {}", e))?;

        self.swarm.listen_on(listen_addr)?;
        println!("📡 Listening on port {}", port);
        Ok(())
    }

    /// Subscribe to a gossipsub topic
    pub fn subscribe(&mut self, topic: &str) -> Result<(), Box<dyn Error>> {
        let topic = gossipsub::IdentTopic::new(topic);
        self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        println!("📢 Subscribed to topic: {}", topic);
        Ok(())
    }

    /// Publish message to topic
    pub fn publish(&mut self, topic: &str, message: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let topic = gossipsub::IdentTopic::new(topic);
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic, message)?;
        Ok(())
    }

    /// Dial a peer by address
    pub fn dial(&mut self, addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm.dial(addr)?;
        Ok(())
    }

    /// Add a bootstrap node to Kademlia
    pub fn add_bootstrap_node(&mut self, peer_id: PeerId, addr: Multiaddr) {
        self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    /// Get connected peers count
    pub fn connected_peers(&self) -> usize {
        self.swarm.connected_peers().count()
    }

    /// Process next network event (call in loop)
    pub async fn next_event(&mut self) -> Option<SwarmEvent<CouncilBehaviourEvent>> {
        self.swarm.select_next_some().await.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_p2p_network_creation() {
        let result = P2PNetwork::new().await;
        assert!(result.is_ok());

        let network = result.unwrap();
        assert_eq!(network.connected_peers(), 0);
    }

    #[tokio::test]
    async fn test_peer_id_generation() {
        let network1 = P2PNetwork::new().await.unwrap();
        let network2 = P2PNetwork::new().await.unwrap();

        // Each instance should have unique peer ID
        assert_ne!(network1.local_peer_id(), network2.local_peer_id());
    }
}

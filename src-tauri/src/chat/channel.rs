use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Channel types in the chat interface
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    /// General discussion, help, status updates
    General,
    /// Human-to-human communication (no AI allowed)
    Human,
    /// Search knowledge bank, view past decisions
    Knowledge,
    /// Submit questions, watch deliberation, see verdicts
    Vote,
}

impl ChannelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChannelType::General => "general",
            ChannelType::Human => "human",
            ChannelType::Knowledge => "knowledge",
            ChannelType::Vote => "vote",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "general" => Some(ChannelType::General),
            "human" => Some(ChannelType::Human),
            "knowledge" => Some(ChannelType::Knowledge),
            "vote" => Some(ChannelType::Vote),
            _ => None,
        }
    }

    /// Check if AI agents can send messages in this channel
    pub fn allows_ai(&self) -> bool {
        match self {
            ChannelType::General => true,
            ChannelType::Human => false, // Human-only channel
            ChannelType::Knowledge => true,
            ChannelType::Vote => true,
        }
    }

    /// Check if this channel requires human signature
    pub fn requires_signature(&self) -> bool {
        matches!(self, ChannelType::Human)
    }
}

/// Message author type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthorType {
    Human,
    AI,
    System,
}

/// Reaction to a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel: ChannelType,
    pub author: String,
    pub author_type: AuthorType,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>, // Ed25519 signature (hex-encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>, // Thread support
    #[serde(default)]
    pub reactions: Vec<Reaction>,
}

impl Message {
    pub fn new(
        channel: ChannelType,
        author: String,
        author_type: AuthorType,
        content: String,
    ) -> Self {
        let timestamp = Utc::now();
        let id = format!("msg_{}", timestamp.timestamp_millis());

        Self {
            id,
            channel,
            author,
            author_type,
            content,
            timestamp,
            signature: None,
            reply_to: None,
            reactions: Vec::new(),
        }
    }

    pub fn system(channel: ChannelType, content: String) -> Self {
        Self::new(channel, "System".to_string(), AuthorType::System, content)
    }

    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }

    pub fn with_reply_to(mut self, reply_to: String) -> Self {
        self.reply_to = Some(reply_to);
        self
    }

    pub fn add_reaction(&mut self, emoji: String, author: String) {
        // Remove existing reaction from same author with same emoji
        self.reactions
            .retain(|r| !(r.emoji == emoji && r.author == author));

        self.reactions.push(Reaction {
            emoji,
            author,
            timestamp: Utc::now(),
        });
    }
}

/// Channel with message history
#[derive(Debug, Clone)]
pub struct Channel {
    pub channel_type: ChannelType,
    pub messages: Vec<Message>,
    pub max_messages: usize,
}

impl Channel {
    pub fn new(channel_type: ChannelType) -> Self {
        Self {
            channel_type,
            messages: Vec::new(),
            max_messages: 10000,
        }
    }

    /// Add a message to the channel
    pub fn add_message(&mut self, message: Message) -> Result<(), String> {
        // Validate channel type matches
        if message.channel != self.channel_type {
            return Err(format!(
                "Message channel {:?} does not match channel {:?}",
                message.channel, self.channel_type
            ));
        }

        // Validate AI messages not allowed in #human
        if !self.channel_type.allows_ai() && message.author_type == AuthorType::AI {
            return Err(format!(
                "AI messages not allowed in #{} channel",
                self.channel_type.as_str()
            ));
        }

        // Validate signature required for #human
        if self.channel_type.requires_signature()
            && message.author_type == AuthorType::Human
            && message.signature.is_none()
        {
            return Err(format!(
                "Signature required for human messages in #{} channel",
                self.channel_type.as_str()
            ));
        }

        self.messages.push(message);

        // Trim old messages if exceeded max
        if self.messages.len() > self.max_messages {
            let excess = self.messages.len() - self.max_messages;
            self.messages.drain(0..excess);
        }

        Ok(())
    }

    /// Get recent messages (newest first)
    pub fn get_messages(&self, limit: usize, offset: usize) -> Vec<Message> {
        let total = self.messages.len();
        if offset >= total {
            return Vec::new();
        }

        let start = total.saturating_sub(offset + limit);
        let end = total.saturating_sub(offset);

        self.messages[start..end].iter().rev().cloned().collect()
    }

    /// Get message by ID
    pub fn get_message(&self, id: &str) -> Option<&Message> {
        self.messages.iter().find(|m| m.id == id)
    }

    /// Get mutable message by ID
    pub fn get_message_mut(&mut self, id: &str) -> Option<&mut Message> {
        self.messages.iter_mut().find(|m| m.id == id)
    }

    /// Get total message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

/// Channel manager for all chat channels
#[derive(Debug, Clone)]
pub struct ChannelManager {
    channels: Arc<Mutex<HashMap<ChannelType, Channel>>>,
}

impl ChannelManager {
    pub fn new() -> Self {
        let mut channels = HashMap::new();
        channels.insert(ChannelType::General, Channel::new(ChannelType::General));
        channels.insert(ChannelType::Human, Channel::new(ChannelType::Human));
        channels.insert(ChannelType::Knowledge, Channel::new(ChannelType::Knowledge));
        channels.insert(ChannelType::Vote, Channel::new(ChannelType::Vote));

        Self {
            channels: Arc::new(Mutex::new(channels)),
        }
    }

    /// Send a message to a channel
    pub fn send_message(&self, message: Message) -> Result<String, String> {
        let mut channels = self.channels.lock().map_err(|e| e.to_string())?;

        let channel = channels
            .get_mut(&message.channel)
            .ok_or_else(|| format!("Channel {:?} not found", message.channel))?;

        let message_id = message.id.clone();
        channel.add_message(message)?;

        Ok(message_id)
    }

    /// Get messages from a channel
    pub fn get_messages(
        &self,
        channel_type: ChannelType,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Message>, String> {
        let channels = self.channels.lock().map_err(|e| e.to_string())?;

        let channel = channels
            .get(&channel_type)
            .ok_or_else(|| format!("Channel {:?} not found", channel_type))?;

        Ok(channel.get_messages(limit, offset))
    }

    /// Get a specific message
    pub fn get_message(
        &self,
        channel_type: ChannelType,
        message_id: &str,
    ) -> Result<Option<Message>, String> {
        let channels = self.channels.lock().map_err(|e| e.to_string())?;

        let channel = channels
            .get(&channel_type)
            .ok_or_else(|| format!("Channel {:?} not found", channel_type))?;

        Ok(channel.get_message(message_id).cloned())
    }

    /// Add reaction to a message
    pub fn add_reaction(
        &self,
        channel_type: ChannelType,
        message_id: &str,
        emoji: String,
        author: String,
    ) -> Result<(), String> {
        let mut channels = self.channels.lock().map_err(|e| e.to_string())?;

        let channel = channels
            .get_mut(&channel_type)
            .ok_or_else(|| format!("Channel {:?} not found", channel_type))?;

        let message = channel
            .get_message_mut(message_id)
            .ok_or_else(|| format!("Message {} not found", message_id))?;

        message.add_reaction(emoji, author);
        Ok(())
    }

    /// Get message count for a channel
    pub fn message_count(&self, channel_type: ChannelType) -> Result<usize, String> {
        let channels = self.channels.lock().map_err(|e| e.to_string())?;

        let channel = channels
            .get(&channel_type)
            .ok_or_else(|| format!("Channel {:?} not found", channel_type))?;

        Ok(channel.message_count())
    }

    /// Send system message to a channel
    pub fn send_system_message(
        &self,
        channel_type: ChannelType,
        content: String,
    ) -> Result<String, String> {
        let message = Message::system(channel_type, content);
        self.send_message(message)
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_type_as_str() {
        assert_eq!(ChannelType::General.as_str(), "general");
        assert_eq!(ChannelType::Human.as_str(), "human");
        assert_eq!(ChannelType::Knowledge.as_str(), "knowledge");
        assert_eq!(ChannelType::Vote.as_str(), "vote");
    }

    #[test]
    fn test_channel_type_from_str() {
        assert_eq!(ChannelType::from_str("general"), Some(ChannelType::General));
        assert_eq!(ChannelType::from_str("human"), Some(ChannelType::Human));
        assert_eq!(
            ChannelType::from_str("knowledge"),
            Some(ChannelType::Knowledge)
        );
        assert_eq!(ChannelType::from_str("vote"), Some(ChannelType::Vote));
        assert_eq!(ChannelType::from_str("invalid"), None);
    }

    #[test]
    fn test_channel_type_allows_ai() {
        assert!(ChannelType::General.allows_ai());
        assert!(!ChannelType::Human.allows_ai()); // Human-only
        assert!(ChannelType::Knowledge.allows_ai());
        assert!(ChannelType::Vote.allows_ai());
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::new(
            ChannelType::General,
            "test_user".to_string(),
            AuthorType::Human,
            "Hello world".to_string(),
        );

        assert_eq!(msg.channel, ChannelType::General);
        assert_eq!(msg.author, "test_user");
        assert_eq!(msg.author_type, AuthorType::Human);
        assert_eq!(msg.content, "Hello world");
        assert!(msg.id.starts_with("msg_"));
        assert!(msg.signature.is_none());
        assert!(msg.reply_to.is_none());
        assert!(msg.reactions.is_empty());
    }

    #[test]
    fn test_system_message() {
        let msg = Message::system(ChannelType::General, "System message".to_string());

        assert_eq!(msg.author, "System");
        assert_eq!(msg.author_type, AuthorType::System);
        assert_eq!(msg.content, "System message");
    }

    #[test]
    fn test_message_with_signature() {
        let msg = Message::new(
            ChannelType::Human,
            "test_user".to_string(),
            AuthorType::Human,
            "Signed message".to_string(),
        )
        .with_signature("abc123".to_string());

        assert_eq!(msg.signature, Some("abc123".to_string()));
    }

    #[test]
    fn test_message_reactions() {
        let mut msg = Message::new(
            ChannelType::General,
            "test_user".to_string(),
            AuthorType::Human,
            "Test".to_string(),
        );

        msg.add_reaction("👍".to_string(), "user1".to_string());
        assert_eq!(msg.reactions.len(), 1);
        assert_eq!(msg.reactions[0].emoji, "👍");
        assert_eq!(msg.reactions[0].author, "user1");

        // Same user, different emoji
        msg.add_reaction("❤️".to_string(), "user1".to_string());
        assert_eq!(msg.reactions.len(), 2);

        // Same user, same emoji (should replace)
        msg.add_reaction("👍".to_string(), "user1".to_string());
        assert_eq!(msg.reactions.len(), 2);
        assert!(msg.reactions.iter().filter(|r| r.emoji == "👍").count() == 1);
    }

    #[test]
    fn test_channel_add_message() {
        let mut channel = Channel::new(ChannelType::General);

        let msg = Message::new(
            ChannelType::General,
            "user1".to_string(),
            AuthorType::Human,
            "Test message".to_string(),
        );

        assert!(channel.add_message(msg).is_ok());
        assert_eq!(channel.message_count(), 1);
    }

    #[test]
    fn test_channel_wrong_type() {
        let mut channel = Channel::new(ChannelType::General);

        let msg = Message::new(
            ChannelType::Vote, // Wrong channel
            "user1".to_string(),
            AuthorType::Human,
            "Test".to_string(),
        );

        assert!(channel.add_message(msg).is_err());
    }

    #[test]
    fn test_channel_human_ai_restriction() {
        let mut channel = Channel::new(ChannelType::Human);

        // Human message should work
        let human_msg = Message::new(
            ChannelType::Human,
            "human_user".to_string(),
            AuthorType::Human,
            "Hello".to_string(),
        )
        .with_signature("sig123".to_string());
        assert!(channel.add_message(human_msg).is_ok());

        // AI message should fail
        let ai_msg = Message::new(
            ChannelType::Human,
            "ai_agent".to_string(),
            AuthorType::AI,
            "Hi".to_string(),
        );
        assert!(channel.add_message(ai_msg).is_err());
    }

    #[test]
    fn test_channel_signature_required() {
        let mut channel = Channel::new(ChannelType::Human);

        // Human message without signature should fail
        let msg = Message::new(
            ChannelType::Human,
            "user1".to_string(),
            AuthorType::Human,
            "Test".to_string(),
        );
        assert!(channel.add_message(msg).is_err());

        // With signature should work
        let msg = Message::new(
            ChannelType::Human,
            "user1".to_string(),
            AuthorType::Human,
            "Test".to_string(),
        )
        .with_signature("sig123".to_string());
        assert!(channel.add_message(msg).is_ok());
    }

    #[test]
    fn test_channel_get_messages() {
        let mut channel = Channel::new(ChannelType::General);

        // Add 5 messages
        for i in 0..5 {
            let msg = Message::new(
                ChannelType::General,
                "user1".to_string(),
                AuthorType::Human,
                format!("Message {}", i),
            );
            channel.add_message(msg).unwrap();
        }

        // Get last 3 messages (newest first)
        let messages = channel.get_messages(3, 0);
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].content, "Message 4"); // Newest
        assert_eq!(messages[2].content, "Message 2"); // Oldest of the 3
    }

    #[test]
    fn test_channel_manager() {
        let manager = ChannelManager::new();

        // Send message to #general
        let msg = Message::new(
            ChannelType::General,
            "user1".to_string(),
            AuthorType::Human,
            "Hello".to_string(),
        );
        let msg_id = manager.send_message(msg).unwrap();

        // Retrieve message
        let messages = manager.get_messages(ChannelType::General, 10, 0).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, msg_id);
        assert_eq!(messages[0].content, "Hello");
    }

    #[test]
    fn test_channel_manager_reactions() {
        let manager = ChannelManager::new();

        // Send message
        let msg = Message::new(
            ChannelType::General,
            "user1".to_string(),
            AuthorType::Human,
            "React to this".to_string(),
        );
        let msg_id = manager.send_message(msg).unwrap();

        // Add reaction
        assert!(manager
            .add_reaction(
                ChannelType::General,
                &msg_id,
                "👍".to_string(),
                "user2".to_string()
            )
            .is_ok());

        // Check reaction
        let msg = manager
            .get_message(ChannelType::General, &msg_id)
            .unwrap()
            .unwrap();
        assert_eq!(msg.reactions.len(), 1);
        assert_eq!(msg.reactions[0].emoji, "👍");
    }

    #[test]
    fn test_channel_manager_system_message() {
        let manager = ChannelManager::new();

        let msg_id = manager
            .send_system_message(ChannelType::General, "Welcome!".to_string())
            .unwrap();

        let messages = manager.get_messages(ChannelType::General, 10, 0).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].author, "System");
        assert_eq!(messages[0].author_type, AuthorType::System);
        assert_eq!(messages[0].content, "Welcome!");
    }
}

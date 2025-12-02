use crate::logger::{LogLevel, Logger};
use crate::providers::{AIProvider, ProviderError};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for managing multiple AI providers
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn AIProvider>>,
    default_generation_provider: Option<String>,
    default_embedding_provider: Option<String>,
    logger: Arc<Logger>,
}

impl ProviderRegistry {
    /// Create new empty registry
    pub fn new(logger: Arc<Logger>) -> Self {
        logger.log(
            LogLevel::Info,
            "registry",
            "📦 Initializing provider registry",
        );

        Self {
            providers: HashMap::new(),
            default_generation_provider: None,
            default_embedding_provider: None,
            logger,
        }
    }

    /// Register a provider with given ID
    pub fn register(&mut self, id: String, provider: Arc<dyn AIProvider>) {
        self.logger.log(
            LogLevel::Info,
            "registry",
            &format!("📦 Registering provider: {} ({})", provider.name(), id),
        );

        self.providers.insert(id, provider);
    }

    /// Get provider by ID
    pub fn get(&self, id: &str) -> Option<Arc<dyn AIProvider>> {
        self.providers.get(id).cloned()
    }

    /// Set default generation provider
    pub fn set_default_generation(&mut self, id: String) -> Result<(), String> {
        if !self.providers.contains_key(&id) {
            return Err(format!("Provider '{}' not found", id));
        }

        self.logger.log(
            LogLevel::Info,
            "registry",
            &format!("🔧 Set default generation provider: {}", id),
        );

        self.default_generation_provider = Some(id);
        Ok(())
    }

    /// Set default embedding provider
    pub fn set_default_embedding(&mut self, id: String) -> Result<(), String> {
        if !self.providers.contains_key(&id) {
            return Err(format!("Provider '{}' not found", id));
        }

        let provider = self.providers.get(&id).unwrap();
        if !provider.supports_embeddings() {
            return Err(format!("Provider '{}' does not support embeddings", id));
        }

        self.logger.log(
            LogLevel::Info,
            "registry",
            &format!("🔧 Set default embedding provider: {}", id),
        );

        self.default_embedding_provider = Some(id);
        Ok(())
    }

    /// Get default generation provider
    pub fn get_generation_provider(&self) -> Result<Arc<dyn AIProvider>, ProviderError> {
        let id = self
            .default_generation_provider
            .as_ref()
            .ok_or(ProviderError::NotSupported(
                "No default generation provider set".to_string(),
            ))?;

        self.get(id).ok_or(ProviderError::NotSupported(format!(
            "Default generation provider '{}' not found",
            id
        )))
    }

    /// Get default embedding provider
    pub fn get_embedding_provider(&self) -> Result<Arc<dyn AIProvider>, ProviderError> {
        let id = self
            .default_embedding_provider
            .as_ref()
            .ok_or(ProviderError::NotSupported(
                "No default embedding provider set".to_string(),
            ))?;

        self.get(id).ok_or(ProviderError::NotSupported(format!(
            "Default embedding provider '{}' not found",
            id
        )))
    }

    /// List all registered provider IDs
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Get provider count
    pub fn count(&self) -> usize {
        self.providers.len()
    }

    /// Check if provider exists
    pub fn has_provider(&self, id: &str) -> bool {
        self.providers.contains_key(id)
    }

    /// Remove provider
    pub fn remove(&mut self, id: &str) -> bool {
        if self.providers.remove(id).is_some() {
            self.logger.log(
                LogLevel::Info,
                "registry",
                &format!("🗑️ Removed provider: {}", id),
            );

            // Clear defaults if they referenced removed provider
            if self.default_generation_provider.as_deref() == Some(id) {
                self.default_generation_provider = None;
            }
            if self.default_embedding_provider.as_deref() == Some(id) {
                self.default_embedding_provider = None;
            }

            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::Logger;
    use crate::providers::ollama::OllamaProvider;

    #[test]
    fn test_registry_creation() {
        let logger = Arc::new(Logger::new(false));
        let registry = ProviderRegistry::new(logger);
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let logger = Arc::new(Logger::new(false));
        let mut registry = ProviderRegistry::new(logger.clone());

        let provider = Arc::new(OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "test".to_string(),
            logger,
        ));

        registry.register("test_provider".to_string(), provider);

        assert_eq!(registry.count(), 1);
        assert!(registry.has_provider("test_provider"));
        assert!(registry.get("test_provider").is_some());
    }

    #[test]
    fn test_default_providers() {
        let logger = Arc::new(Logger::new(false));
        let mut registry = ProviderRegistry::new(logger.clone());

        let provider = Arc::new(OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "test".to_string(),
            logger,
        ));

        registry.register("test_provider".to_string(), provider);

        // Set defaults
        assert!(registry
            .set_default_generation("test_provider".to_string())
            .is_ok());
        assert!(registry
            .set_default_embedding("test_provider".to_string())
            .is_ok());

        // Get defaults
        assert!(registry.get_generation_provider().is_ok());
        assert!(registry.get_embedding_provider().is_ok());
    }

    #[test]
    fn test_remove_provider() {
        let logger = Arc::new(Logger::new(false));
        let mut registry = ProviderRegistry::new(logger.clone());

        let provider = Arc::new(OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "test".to_string(),
            logger,
        ));

        registry.register("test_provider".to_string(), provider);
        assert!(registry.remove("test_provider"));
        assert!(!registry.has_provider("test_provider"));
        assert_eq!(registry.count(), 0);
    }
}

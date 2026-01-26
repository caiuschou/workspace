//! OAuth client storage interface.
//!
//! This module defines the trait for storing and retrieving OAuth client information.

use async_trait::async_trait;

use mcp_core::auth::{OAuthClientInformation, OAuthClientInformationFull, OAuthClientMetadata};

/// Error type for client store operations.
#[derive(Debug, thiserror::Error)]
pub enum ClientStoreError {
    /// Client not found.
    #[error("client not found: {0}")]
    NotFound(String),

    /// Invalid client metadata.
    #[error("invalid client metadata: {0}")]
    InvalidMetadata(String),

    /// Storage error.
    #[error("storage error: {0}")]
    Storage(String),
}

/// Trait for storing and retrieving registered OAuth clients.
#[async_trait]
pub trait OAuthRegisteredClientsStore: Send + Sync {
    /// Get a client by its ID.
    async fn get_client(&self, client_id: &str) -> Result<Option<OAuthClientInformationFull>, ClientStoreError>;

    /// Register a new client.
    ///
    /// Returns the full client information including the generated client_id and optional client_secret.
    async fn register_client(&self, metadata: OAuthClientMetadata) -> Result<OAuthClientInformationFull, ClientStoreError>;

    /// Update an existing client.
    async fn update_client(&self, client_id: &str, metadata: OAuthClientMetadata) -> Result<OAuthClientInformationFull, ClientStoreError> {
        // Default implementation: not supported
        let _ = (client_id, metadata);
        Err(ClientStoreError::Storage("update not supported".to_string()))
    }

    /// Delete a client.
    async fn delete_client(&self, client_id: &str) -> Result<(), ClientStoreError> {
        // Default implementation: not supported
        let _ = client_id;
        Err(ClientStoreError::Storage("delete not supported".to_string()))
    }
}

/// In-memory implementation of the client store for development/testing.
pub struct InMemoryClientStore {
    clients: std::sync::RwLock<std::collections::HashMap<String, OAuthClientInformationFull>>,
}

impl InMemoryClientStore {
    /// Create a new in-memory client store.
    pub fn new() -> Self {
        Self {
            clients: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Add a pre-registered client.
    pub fn add_client(&self, client: OAuthClientInformationFull) {
        let mut clients = self.clients.write().unwrap();
        clients.insert(client.client_info.client_id.clone(), client);
    }

    /// Generate a unique client ID.
    fn generate_client_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Generate a client secret.
    fn generate_client_secret() -> String {
        // In production, use a cryptographically secure random generator
        uuid::Uuid::new_v4().to_string().replace("-", "")
    }
}

impl Default for InMemoryClientStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OAuthRegisteredClientsStore for InMemoryClientStore {
    async fn get_client(&self, client_id: &str) -> Result<Option<OAuthClientInformationFull>, ClientStoreError> {
        let clients = self.clients.read().unwrap();
        Ok(clients.get(client_id).cloned())
    }

    async fn register_client(&self, metadata: OAuthClientMetadata) -> Result<OAuthClientInformationFull, ClientStoreError> {
        // Validate metadata
        if metadata.redirect_uris.is_empty() {
            return Err(ClientStoreError::InvalidMetadata(
                "redirect_uris is required".to_string(),
            ));
        }

        let client_id = Self::generate_client_id();
        let client_secret = Self::generate_client_secret();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let client = OAuthClientInformationFull {
            client_info: OAuthClientInformation {
                client_id: client_id.clone(),
                client_secret: Some(client_secret),
                client_id_issued_at: Some(now),
                client_secret_expires_at: None, // Non-expiring
            },
            metadata,
            token_endpoint_auth_method: Some("client_secret_post".to_string()),
        };

        let mut clients = self.clients.write().unwrap();
        clients.insert(client_id, client.clone());

        Ok(client)
    }

    async fn update_client(&self, client_id: &str, metadata: OAuthClientMetadata) -> Result<OAuthClientInformationFull, ClientStoreError> {
        let mut clients = self.clients.write().unwrap();

        if let Some(existing) = clients.get_mut(client_id) {
            existing.metadata = metadata;
            Ok(existing.clone())
        } else {
            Err(ClientStoreError::NotFound(client_id.to_string()))
        }
    }

    async fn delete_client(&self, client_id: &str) -> Result<(), ClientStoreError> {
        let mut clients = self.clients.write().unwrap();
        if clients.remove(client_id).is_some() {
            Ok(())
        } else {
            Err(ClientStoreError::NotFound(client_id.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_store_register() {
        let store = InMemoryClientStore::new();

        let metadata = OAuthClientMetadata {
            redirect_uris: vec!["http://localhost:8080/callback".to_string()],
            client_name: Some("Test Client".to_string()),
            ..Default::default()
        };

        let client = store.register_client(metadata).await.unwrap();

        assert!(!client.client_info.client_id.is_empty());
        assert!(client.client_info.client_secret.is_some());
        assert_eq!(client.metadata.client_name, Some("Test Client".to_string()));
    }

    #[tokio::test]
    async fn test_in_memory_store_get() {
        let store = InMemoryClientStore::new();

        let metadata = OAuthClientMetadata {
            redirect_uris: vec!["http://localhost:8080/callback".to_string()],
            ..Default::default()
        };

        let registered = store.register_client(metadata).await.unwrap();
        let retrieved = store.get_client(&registered.client_info.client_id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().client_info.client_id, registered.client_info.client_id);
    }

    #[tokio::test]
    async fn test_in_memory_store_not_found() {
        let store = InMemoryClientStore::new();

        let result = store.get_client("nonexistent").await.unwrap();
        assert!(result.is_none());
    }
}

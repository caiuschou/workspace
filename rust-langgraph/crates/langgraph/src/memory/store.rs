//! Store trait and StoreError for cross-thread memory.
//!
//! Aligns with LangGraph BaseStore (namespace, put, get, list, search). See docs/rust-langgraph/16-memory-design.md §5.1.

use async_trait::async_trait;

/// Namespace for Store: e.g. (user_id, "memories") or (user_id, "preferences"). Aligns with LangGraph namespace tuple.
pub type Namespace = Vec<String>;

/// Error for store operations.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("serialization: {0}")]
    Serialization(String),
    #[error("storage: {0}")]
    Storage(String),
    #[error("not found")]
    NotFound,
}

/// Hit returned by Store::search (key, value, optional score).
#[derive(Debug, Clone)]
pub struct StoreSearchHit {
    pub key: String,
    pub value: serde_json::Value,
    pub score: Option<f64>,
}

/// Cross-thread key-value and optional semantic search.
///
/// Aligns with LangGraph BaseStore. Implementations: InMemoryStore (in-memory; semantic search via in-memory vector store).
#[async_trait]
pub trait Store: Send + Sync {
    /// Put a value under namespace and key.
    async fn put(
        &self,
        namespace: &Namespace,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<(), StoreError>;

    /// Get value by namespace and key.
    async fn get(
        &self,
        namespace: &Namespace,
        key: &str,
    ) -> Result<Option<serde_json::Value>, StoreError>;

    /// List keys in namespace.
    async fn list(&self, namespace: &Namespace) -> Result<Vec<String>, StoreError>;

    /// Search in namespace. With semantic index: query is natural language; otherwise key prefix or full scan.
    async fn search(
        &self,
        namespace: &Namespace,
        query: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<StoreSearchHit>, StoreError>;
}

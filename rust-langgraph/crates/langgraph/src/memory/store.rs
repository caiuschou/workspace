//! Store trait and StoreError for cross-thread memory.
//!
//! Aligns with LangGraph BaseStore (namespace, put, get, list, search). See docs/rust-langgraph/16-memory-design.md §5.1.

use async_trait::async_trait;

/// Namespace for Store: e.g. (user_id, "memories") or (user_id, "preferences"). Aligns with LangGraph namespace tuple.
pub type Namespace = Vec<String>;

/// Error for store operations.
///
/// Callers do not depend on underlying backend errors (e.g. rusqlite, lancedb).
/// Use `?` with `serde_json::Error` via the `From` impl for serialization failures.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// JSON or namespace serialization/deserialization failed.
    #[error("serialization: {0}")]
    Serialization(String),

    /// Backend storage error (e.g. DB I/O). Message is opaque to avoid leaking backend types.
    #[error("storage: {0}")]
    Storage(String),

    /// Key not found in the given namespace (optional; get may use `Ok(None)` instead).
    #[error("not found")]
    NotFound,
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        StoreError::Serialization(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_error_from_serde_json_error() {
        let invalid = "not valid json {{{";
        let err: StoreError = serde_json::from_str::<serde_json::Value>(invalid).unwrap_err().into();
        match &err {
            StoreError::Serialization(s) => assert!(s.contains("expected value") || s.len() > 0),
            _ => panic!("expected Serialization variant"),
        }
    }
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

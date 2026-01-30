//! Memory: config, checkpoint, checkpointer, store.
//!
//! Aligns with LangGraph Checkpointer + Store. Design: docs/rust-langgraph/16-memory-design.md.

mod checkpoint;
mod checkpointer;
mod config;
#[cfg(feature = "lance")]
mod embedder;
mod in_memory_store;
mod memory_saver;
mod serializer;
mod store;

#[cfg(feature = "lance")]
mod lance_store;
#[cfg(feature = "sqlite")]
mod sqlite_saver;
#[cfg(feature = "sqlite")]
mod sqlite_store;

pub use checkpoint::{Checkpoint, CheckpointListItem, CheckpointMetadata, CheckpointSource};
pub use checkpointer::{CheckpointError, Checkpointer};
pub use config::RunnableConfig;
pub use in_memory_store::InMemoryStore;
pub use memory_saver::MemorySaver;
pub use serializer::{JsonSerializer, Serializer};
pub use store::{Namespace, Store, StoreError, StoreSearchHit};

#[cfg(feature = "lance")]
pub use embedder::Embedder;
#[cfg(feature = "lance")]
pub use lance_store::LanceStore;
#[cfg(feature = "sqlite")]
pub use sqlite_saver::SqliteSaver;
#[cfg(feature = "sqlite")]
pub use sqlite_store::SqliteStore;

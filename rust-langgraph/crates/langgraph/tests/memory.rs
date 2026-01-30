//! Unit tests for memory: RunnableConfig, Checkpoint, MemorySaver, Checkpointer, Store. Design: 16-memory-design.md.

use langgraph::memory::{
    Checkpoint, CheckpointMetadata, CheckpointSource, Checkpointer, InMemoryStore, MemorySaver,
    RunnableConfig,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
struct TestState {
    value: String,
}

#[tokio::test]
async fn memory_saver_put_and_get_tuple() {
    let saver: MemorySaver<TestState> = MemorySaver::new();
    let config = RunnableConfig {
        thread_id: Some("t1".into()),
        checkpoint_id: None,
        checkpoint_ns: String::new(),
        user_id: None,
    };
    let checkpoint = Checkpoint {
        id: "c1".into(),
        ts: "123".into(),
        channel_values: TestState {
            value: "hello".into(),
        },
        channel_versions: HashMap::new(),
        metadata: CheckpointMetadata {
            source: CheckpointSource::Update,
            step: 0,
            created_at: None,
        },
    };
    let id = saver.put(&config, &checkpoint).await.unwrap();
    assert_eq!(id, "c1");

    let tuple = saver.get_tuple(&config).await.unwrap();
    let (cp, _meta) = tuple.unwrap();
    assert_eq!(cp.id, "c1");
    assert_eq!(cp.channel_values.value, "hello");
}

#[tokio::test]
async fn memory_saver_get_tuple_empty_returns_none() {
    let saver: MemorySaver<TestState> = MemorySaver::new();
    let config = RunnableConfig {
        thread_id: Some("t2".into()),
        checkpoint_id: None,
        checkpoint_ns: String::new(),
        user_id: None,
    };
    let tuple = saver.get_tuple(&config).await.unwrap();
    assert!(tuple.is_none());
}

#[tokio::test]
async fn memory_saver_list_returns_empty_when_no_checkpoints() {
    let saver: MemorySaver<TestState> = MemorySaver::new();
    let config = RunnableConfig {
        thread_id: Some("t3".into()),
        checkpoint_id: None,
        checkpoint_ns: String::new(),
        user_id: None,
    };
    let list = saver.list(&config, None, None, None).await.unwrap();
    assert!(list.is_empty());
}

#[tokio::test]
async fn checkpoint_from_state() {
    let state = TestState {
        value: "test".into(),
    };
    let cp = Checkpoint::from_state(state, CheckpointSource::Update, 1);
    assert!(!cp.id.is_empty());
    assert!(!cp.ts.is_empty());
    assert_eq!(cp.channel_values.value, "test");
    assert_eq!(cp.metadata.step, 1);
    assert!(cp.channel_versions.is_empty());
}

#[tokio::test]
async fn in_memory_store_put_get_list_search() {
    use langgraph::memory::Store;
    use serde_json::json;

    let store = InMemoryStore::new();
    let ns = vec!["user1".into(), "memories".into()];

    store
        .put(&ns, "k1", &json!("v1"))
        .await
        .unwrap();
    store
        .put(&ns, "k2", &json!({"x": 1}))
        .await
        .unwrap();

    let v = store.get(&ns, "k1").await.unwrap();
    assert_eq!(v, Some(json!("v1")));

    let keys = store.list(&ns).await.unwrap();
    assert!(keys.contains(&"k1".into()));
    assert!(keys.contains(&"k2".into()));

    let hits = store.search(&ns, Some("v1"), None).await.unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].key, "k1");
}

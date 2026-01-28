//! 记忆扩展示例：SessionMemory + VectorMemory + MockEmbedder。
//!
//! 运行：`cargo run -p langgraph --example memory`
//! 演示会话记忆追加消息、向量记忆按相似度检索（MockEmbedder 生成伪向量）。

use langgraph::{
    Embedder, Memory, Message, MockEmbedder, SemanticMemory, SessionMemory, VectorMemory,
};

fn main() {
    println!("=== SessionMemory ===");
    let session = SessionMemory::with_capacity(5);
    session.add(Message::user("第一句"));
    session.add(Message::assistant("第二句"));
    session.add(Message::user("第三句"));
    let got = session.get(10);
    assert_eq!(got.len(), 3);
    println!("messages: {}", got.iter().map(|m| m.content.as_str()).collect::<Vec<_>>().join(", "));

    println!("\n=== VectorMemory + MockEmbedder ===");
    let embedder = MockEmbedder::new(8);
    let vec_mem = VectorMemory::new();
    vec_mem.add("Rust is a systems programming language.", embedder.embed("Rust is a systems programming language.").unwrap().as_slice());
    vec_mem.add("Python is great for scripting.", embedder.embed("Python is great for scripting.").unwrap().as_slice());
    vec_mem.add("Memory can be session-based or vector-based.", embedder.embed("Memory can be session-based or vector-based.").unwrap().as_slice());

    let query = "What is Rust?";
    let query_vec = embedder.embed(query).unwrap();
    let results = vec_mem.search(&query_vec, 2);
    println!("query: \"{}\"", query);
    for r in &results {
        println!("  score={:.4} content={}", r.score, r.content);
    }
    println!("memory example ok");
}

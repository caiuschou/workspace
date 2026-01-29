//! 向量语义记忆：按相似度检索。
//!
//! - `SemanticMemory`: add(content, embedding)、search(query_embedding, top_k)
//! - `VectorMemory`: 实现 SemanticMemory，存 MemoryEmbedding，余弦相似度 search
//! - `MemoryResult`: 检索结果（id, content, score）

use std::sync::RwLock;

/// 单条检索结果：内容与相似度分数。
///
/// Returned by `SemanticMemory::search`. `score` is cosine similarity in [0, 1] when
/// vectors are normalized, or raw similarity depending on implementation.
#[derive(Debug, Clone)]
pub struct MemoryResult {
    /// 可选 id，与存储时一致。
    pub id: Option<String>,
    /// 存储的文本内容。
    pub content: String,
    /// 相似度分数，越高越相关。
    pub score: f32,
    /// 可选元数据。
    pub metadata: Option<serde_json::Value>,
}

/// 单条向量记忆：id、内容、向量与可选元数据。
///
/// Stored by `VectorMemory`. Used internally for cosine similarity in `search`.
#[derive(Debug, Clone)]
pub struct MemoryEmbedding {
    /// 唯一 id。
    pub id: String,
    /// 文本内容。
    pub content: String,
    /// 向量，长度需与 query 一致。
    pub vector: Vec<f32>,
    /// 可选元数据。
    pub metadata: Option<serde_json::Value>,
}

/// 语义记忆 trait：按向量存储并相似度检索。
///
/// Used by RAG or MemoryAgent to inject related past content into the prompt.
/// Implemented by `VectorMemory`; callers use `Embedder` to get vectors for `add` and for query.
pub trait SemanticMemory: Send + Sync {
    /// 存储一条内容与其向量。
    fn add(&self, content: &str, embedding: &[f32]);

    /// 按查询向量检索最相似的 `top_k` 条，按分数降序。
    fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<MemoryResult>;
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    (dot / (na * nb)).clamp(-1.0, 1.0)
}

/// 向量记忆：内存中存储 `MemoryEmbedding`，按余弦相似度检索。
///
/// Implements `SemanticMemory`. Callers typically use an `Embedder` to produce
/// embeddings for `add` and for the query in `search`.
pub struct VectorMemory {
    inner: RwLock<Vec<MemoryEmbedding>>,
    next_id: RwLock<u64>,
}

impl VectorMemory {
    /// 新建空向量记忆。
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Vec::new()),
            next_id: RwLock::new(0),
        }
    }
}

impl Default for VectorMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticMemory for VectorMemory {
    fn add(&self, content: &str, embedding: &[f32]) {
        let mut id_guard = self.next_id.write().unwrap();
        let id = format!("{}", *id_guard);
        *id_guard += 1;
        drop(id_guard);
        let entry = MemoryEmbedding {
            id: id.clone(),
            content: content.to_string(),
            vector: embedding.to_vec(),
            metadata: None,
        };
        self.inner.write().unwrap().push(entry);
    }

    fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<MemoryResult> {
        let guard = self.inner.read().unwrap();
        let mut scored: Vec<(f32, &MemoryEmbedding)> = guard
            .iter()
            .map(|e| (cosine_similarity(&e.vector, query_embedding), e))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(top_k)
            .map(|(score, e)| MemoryResult {
                id: Some(e.id.clone()),
                content: e.content.clone(),
                score,
                metadata: e.metadata.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_memory_add_search() {
        let m = VectorMemory::new();
        let v1 = vec![1.0, 0.0, 0.0f32];
        let v2 = vec![0.9, 0.1, 0.0f32];
        let vq = vec![1.0, 0.0, 0.0f32];
        m.add("a", &v1);
        m.add("b", &v2);
        let out = m.search(&vq, 2);
        assert_eq!(out.len(), 2);
        assert!(out[0].score >= out[1].score);
        assert_eq!(out[0].content, "a");
    }
}

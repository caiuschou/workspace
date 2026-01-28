//! 文本向量化接口，用于语义记忆与 RAG。
//!
//! - `Embedder`: embed(text)、embed_batch(texts)
//! - `MockEmbedder`: 测试用，返回固定维度的确定性向量

/// 文本向量化 trait：将文本转为浮点向量，供 `VectorMemory` 等使用。
///
/// Used by `VectorMemory` (or callers) to obtain embeddings for `SemanticMemory::add`
/// and for query in `search`. Implementations: `MockEmbedder` (tests), `OpenAiEmbedder` (feature openai).
pub trait Embedder: Send + Sync {
    /// 单条文本的向量维度。
    fn dimension(&self) -> usize;

    /// 将一条文本转为向量。
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedderError>;

    /// 将多条文本转为向量；默认实现逐条调用 `embed`。
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbedderError> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
}

/// 向量化失败错误。
#[derive(Debug, Clone)]
pub struct EmbedderError(pub String);

impl std::fmt::Display for EmbedderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "embedder error: {}", self.0)
    }
}

impl std::error::Error for EmbedderError {}

/// Mock 向量化：根据文本生成固定维度的确定性向量，仅用于测试。
///
/// Uses a simple hash-based pseudo-embedding so same text yields same vector.
/// Used by tests and by examples when no real embedder is configured.
#[derive(Debug, Clone)]
pub struct MockEmbedder {
    dimension: usize,
}

impl MockEmbedder {
    /// 新建 mock，向量维度为 `dimension`。
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }

    /// 默认维度 8。
    pub fn default_dimension() -> Self {
        Self::new(8)
    }
}

impl Embedder for MockEmbedder {
    fn dimension(&self) -> usize {
        self.dimension
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedderError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut out = Vec::with_capacity(self.dimension);
        for i in 0..self.dimension {
            let mut h = DefaultHasher::new();
            text.as_bytes().hash(&mut h);
            (i as u64).hash(&mut h);
            let v = (h.finish() % 1000) as f32 / 1000.0;
            out.push(v);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_embedder_same_text_same_vec() {
        let e = MockEmbedder::new(4);
        let a = e.embed("hello").unwrap();
        let b = e.embed("hello").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn mock_embedder_dimension() {
        let e = MockEmbedder::new(16);
        let v = e.embed("x").unwrap();
        assert_eq!(v.len(), 16);
    }
}

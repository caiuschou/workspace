//! 会话记忆：FIFO 容量限制的进程内消息列表。

use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use crate::message::Message;
use super::Memory;

/// 会话记忆：基于 `Vec<Message>` 的 FIFO 存储，带容量上限。
#[derive(Debug, Clone)]
pub struct SessionMemory {
    inner: Arc<RwLock<Vec<Message>>>,
    /// 最大保留条数，超出时从头部删除。
    capacity: usize,
}

impl SessionMemory {
    /// 新建无容量限制的会话记忆。
    pub fn new() -> Self {
        Self::with_capacity(usize::MAX)
    }

    /// 新建最多保留 `capacity` 条的会话记忆。
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Vec::new())),
            capacity,
        }
    }

    fn truncate_to_capacity(vec: &mut Vec<Message>, cap: usize) {
        if vec.len() > cap {
            vec.drain(..vec.len() - cap);
        }
    }
}

impl Default for SessionMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory for SessionMemory {
    fn add(&self, msg: Message) {
        let msg = if msg.timestamp.is_none() {
            msg.with_timestamp(SystemTime::now())
        } else {
            msg
        };
        let mut g = self.inner.write().expect("session memory lock");
        g.push(msg);
        Self::truncate_to_capacity(&mut g, self.capacity);
    }

    fn get(&self, limit: usize) -> Vec<Message> {
        let g = self.inner.read().expect("session memory lock");
        let start = g.len().saturating_sub(limit);
        g[start..].to_vec()
    }

    fn clear(&self) {
        self.inner.write().expect("session memory lock").clear();
    }

    fn count(&self) -> usize {
        self.inner.read().expect("session memory lock").len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Message;

    #[test]
    fn session_add_get() {
        let m = SessionMemory::with_capacity(10);
        m.add(Message::user("a"));
        m.add(Message::assistant("b"));
        assert_eq!(m.count(), 2);
        let g = m.get(2);
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].content, "a");
        assert_eq!(g[1].content, "b");
    }

    #[test]
    fn session_capacity() {
        let m = SessionMemory::with_capacity(2);
        m.add(Message::user("1"));
        m.add(Message::assistant("2"));
        m.add(Message::user("3"));
        assert_eq!(m.count(), 2);
        let g = m.get(10);
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].content, "2");
        assert_eq!(g[1].content, "3");
    }

    #[test]
    fn session_clear() {
        let m = SessionMemory::new();
        m.add(Message::user("x"));
        m.clear();
        assert_eq!(m.count(), 0);
        assert!(m.get(10).is_empty());
    }
}

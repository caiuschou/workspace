//! 记忆类型与会话记忆。
//!
//! - `Memory`: 消息存储 trait（add/get/clear/count）
//! - `SessionMemory`: FIFO 容量限制的进程内会话记忆（S3）

mod session;

use crate::message::Message;

pub use session::SessionMemory;

/// 会话记忆 trait：按顺序追加消息，支持按条数读取与清空。
pub trait Memory: Send + Sync {
    /// 追加一条消息；实现可选择填充时间戳等。
    fn add(&self, msg: Message);

    /// 取最近 `limit` 条消息（FIFO 顺序，最新在末尾）。
    fn get(&self, limit: usize) -> Vec<Message>;

    /// 清空全部消息。
    fn clear(&self);

    /// 当前消息条数。
    fn count(&self) -> usize;
}

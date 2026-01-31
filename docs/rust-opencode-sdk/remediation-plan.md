# opencode-sdk 整改方案

> 基于架构评审的详细整改计划。执行时按优先级顺序推进，完成后在任务表中标记。

## 1. 概述

本方案针对 opencode-sdk 架构评审中发现的问题，按优先级拆分为 **P0（高）**、**P1（中）**、**P2（低）** 三个层级，并给出具体实施步骤。

### 1.1 整改原则

- 简单优先，避免过度设计
- 每个任务可独立完成、可验证
- 遵循 AGENTS.md：一个类型一个文件、测试独立、BDD 风格
- 保持向后兼容，如需 breaking change 需在文档中明确

---

## 2. 任务总览

### 2.1 扁平任务表（按执行顺序）

| 编号 | 任务 | 父任务 | 优先级 | 状态 | 预估 |
|------|------|--------|--------|------|------|
| T1.1 | log.rs：修改 init_logger 签名为返回 Option\<Guard\> | T1 | P0 | 完成 | 15min |
| T1.2 | log.rs：移除 static mut 与 unsafe 块 | T1 | P0 | 完成 | 10min |
| T1.3 | 查找并更新所有 init_logger 调用点 | T1 | P0 | 完成 | 10min |
| T2.1 | 创建 tests 目录与 common.rs | T2 | P0 | 完成 | 15min |
| T2.2 | session.rs：parse_message_list/parse_message_item 改为 pub(crate) | T2 | P0 | 完成 | 5min |
| T2.3 | event.rs：extract_text_delta 改为 pub(crate) | T2 | P0 | 完成 | 5min |
| T2.4 | 编写 detect_test.rs（绝对路径用例） | T2 | P0 | 完成 | 20min |
| T2.5 | 编写 message_parse_test.rs（置于 session mod tests） | T2 | P0 | 完成 | 25min |
| T2.6 | 编写 event_extract_test（置于 event mod tests） | T2 | P0 | 完成 | 20min |
| T2.7 | 运行 cargo test 并确保通过 | T2 | P0 | 完成 | 10min |
| T3.1 | open.rs：抽取 log_assistant_reply 函数 | T3 | P1 | 完成 | 25min |
| T3.2 | open.rs：抽取 run_with_stream 函数 | T3 | P1 | 完成 | 30min |
| T3.3 | open.rs：抽取 run_without_stream 函数 | T3 | P1 | 完成 | 15min |
| T3.4 | open.rs：精简 maybe_send_chat 为协调逻辑 | T3 | P1 | 完成 | 20min |
| T3.5 | 运行 cargo build 与 example 验证 | T3 | P1 | 完成 | 10min |
| T4.1 | open.rs：添加常量并替换 magic number | T4 | P1 | 完成 | 15min |
| T5.1 | event.rs：Err 分支增加 debug 日志 | T5 | P1 | 完成 | 5min |
| T6.1 | open.rs：为 stream_output abort 逻辑添加注释 | T6 | P1 | 完成 | 10min |
| T6.2 | open.rs：（可选）abort 后增加短延时 | T6 | P1 | 跳过 | 5min |
| T7.1 | 新建 session/message.rs，迁移 parse_message_list | T7 | P2 | 完成 | 25min |
| T7.2 | session/message.rs：迁移 parse_message_item | T7 | P2 | 完成 | 15min |
| T7.3 | session/mod.rs：改为调用 message::parse_message_list | T7 | P2 | 完成 | 10min |
| T7.4 | 调整 log_part_received 归属（可选） | T7 | P2 | 跳过 | 15min |
| T8.1 | open.rs：在 open 顶部创建 ReqwestClient | T8 | P2 | 完成 | 10min |
| T8.2 | open.rs：修改 check_server_healthy 签名并传入 client | T8 | P2 | 完成 | 15min |
| T9.1 | error.rs：新增 ClientBuildFailed 变体 | T9 | P2 | 待办 | 5min |
| T9.2 | client.rs：build() 改为返回 Result | T9 | P2 | 待办 | 15min |
| T9.3 | 更新所有 build() 调用点 | T9 | P2 | 待办 | 15min |
| T10.1 | 更新 open-method-design.md 中 shutdown 签名 | T10 | P2 | 完成 | 15min |
| T10.2 | 补充 best-effort 设计说明 | T10 | P2 | 完成 | 10min |

### 2.2 原任务与子步骤对应

| 原任务 | 子步骤 |
|--------|--------|
| T1 | T1.1, T1.2, T1.3 |
| T2 | T2.1 ~ T2.7 |
| T3 | T3.1 ~ T3.5 |
| T4 | T4.1 |
| T5 | T5.1 |
| T6 | T6.1, T6.2 |
| T7 | T7.1 ~ T7.4 |
| T8 | T8.1, T8.2 |
| T9 | T9.1 ~ T9.3 |
| T10 | T10.1, T10.2 |

---

## 3. P0 高优先级任务

### T1: log.rs 移除 static mut（T1.1 ~ T1.3）

**问题**：`LOG_FILE_GUARD` 使用 `static mut` 持有 `WorkerGuard`，违反 Rust 安全规则，且 `LOG_FILE_GUARD` 从未被 `drop`，程序退出时可能产生未定义行为。

**文件**：`opencode-sdk/src/log.rs`

**方案 A（推荐）**：返回 guard，由调用方持有。

```rust
/// Returns the guard. Caller must hold it to keep file logging active.
/// Dropping the guard stops the file writer.
pub fn init_logger(log_dir: Option<PathBuf>) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    // ... 现有逻辑 ...
    // 移除 static mut 和 unsafe 块
    Some(guard)
}
```

**影响**：调用 `init_logger` 的代码需要保存返回值。检查 examples 和 lib 中是否调用。

**子步骤**：
- **T1.1**：修改 `init_logger` 签名为 `-> Option<WorkerGuard>`，返回 `Some(guard)`
- **T1.2**：删除 `static mut LOG_FILE_GUARD` 和 `unsafe { ... }` 块
- **T1.3**：`grep -r init_logger` 查找调用点，更新为 `let _guard = init_logger(...)`；在 doc 中说明可忽略返回值

---

### T2: 单元测试（T2.1 ~ T2.7）

**问题**：无单元测试，不符合 AGENTS.md 要求，且纯函数易测。

**目录**：`opencode-sdk/tests/`（集成测试）或 `opencode-sdk/src/xxx.rs` 内 `#[cfg(test)] mod tests`（单元测试）

**注意**：Rust 的 `tests/` 为独立 crate，只能访问 `pub` 项。若在 `tests/` 中测解析函数，需改为 `pub`（可加 `#[doc(hidden)]`）；否则将解析相关的单元测试放在 `src/session.rs`、`src/event.rs` 的 `#[cfg(test)] mod tests` 中，`tests/` 仅做集成测试。

按 AGENTS.md，单元测试拆分到单独目录，BDD 风格。建议结构：

```
opencode-sdk/
├── src/
└── tests/
    ├── common.rs          # 共享 fixture
    ├── detect_test.rs     # detect_command
    ├── message_parse_test.rs  # parse_message_list, parse_message_item
    └── event_extract_test.rs  # extract_text_delta（需 pub(crate)）
```

**T2.1 detect_command 测试**

- **文件**：`opencode-sdk/tests/detect_test.rs`
- **用例**：
  - 绝对路径存在 → available=true, path=该路径
  - 绝对路径不存在 → available=false
  - `opencode` 在 PATH 中 → available=true, path=首个匹配
  - 不存在的命令 → available=false
- **注意**：`which opencode` 结果依赖环境，可 mock 或仅测绝对路径逻辑

**T2.2 parse_message_list / parse_message_item 测试**

- **文件**：`opencode-sdk/tests/message_parse_test.rs`
- **前置**：需将 `parse_message_list`、`parse_message_item` 改为 `pub(crate)`，或通过 `session::Client::session_list_messages` 间接测（后者依赖 HTTP，更适合集成测试）
- **建议**：在 `session.rs` 中将解析函数标为 `pub(crate)`，供测试模块调用
- **用例**：
  - `{ "messages": [...] }` 格式
  - 顶层数组格式
  - 空对象 / 空数组
  - 单条消息含 `info`、`parts`
  - 单条消息无 `info`，有顶层 `id`、`role`

**T2.3 extract_text_delta 测试**

- **文件**：`opencode-sdk/tests/event_extract_test.rs`
- **前置**：`extract_text_delta` 改为 `pub(crate)`
- **用例**：
  - session_id 匹配，`properties.text` 有值
  - session_id 不匹配 → None
  - `properties.content` 作为 text
  - 空字符串 → None

**子步骤**：
- **T2.1**：创建 `opencode-sdk/tests/` 目录，新建 `common.rs`（可为空或放 shared helper）
- **T2.2**：`session.rs` 中 `parse_message_list`、`parse_message_item` 改为 `pub(crate)`（若用 `tests/` 目录则需 `pub`，因 integration tests 为独立 crate）
- **T2.3**：`event.rs` 中 `extract_text_delta` 改为 `pub(crate)`（同上）
- **T2.4**：新建 `tests/detect_test.rs`，编写绝对路径存在/不存在的用例；`which` 用例视环境选做
- **T2.5**：新建 `tests/message_parse_test.rs`，编写 wrapped/array 格式、空、info 有无等用例
- **T2.6**：新建 `tests/event_extract_test.rs`，编写 session 匹配、text/content、空串等用例
- **T2.7**：`cargo test` 全绿

---

## 4. P1 中优先级任务

### T3: open.rs 拆分 maybe_send_chat（T3.1 ~ T3.5）

**问题**：`maybe_send_chat` 约 200 行，混合会话创建、发消息、等待、流式输出、大量 part 日志，职责过多。

**文件**：`opencode-sdk/src/open.rs`

**拆分方案**：

```
maybe_send_chat (主流程，~30 行)
├── create_session_and_send(client, dir, content, ...) -> (Session, ())
├── wait_for_assistant_response (已有，保持)
├── run_stream_output(client, dir, session_id, content, on_text, wait_ms) -> Option<MessageListItem>
└── log_assistant_reply(reply)  // 将 363-399 行抽出
```

**子步骤**：
- **T3.1**：新增 `fn log_assistant_reply(reply: &MessageListItem)`，将 part 遍历与 `info!` 日志移入
- **T3.2**：新增 `async fn run_with_stream(...)`，内部 spawn event、发消息、`wait_for_assistant_response`、`event_handle.abort`
- **T3.3**：新增 `async fn run_without_stream(...)`，发消息后若 `wait_ms > 0` 则 `wait_for_assistant_response`
- **T3.4**：`maybe_send_chat` 简化为：判断 content、创建 session、按 `stream_output` 调 `run_with_stream` 或 `run_without_stream`、调 `log_assistant_reply`、返回
- **T3.5**：`cargo build`、`cargo run --example open` 验证

---

### T4: 常量提取（T4.1）

**问题**：500、2000 等 magic number 散落，可读性和可调性差。

**文件**：`opencode-sdk/src/open.rs`

**常量表**：

| 常量名 | 值 | 用途 |
|--------|-----|------|
| `SERVER_POLL_INTERVAL_MS` | 500 | 服务就绪轮询间隔 |
| `MESSAGE_POLL_INTERVAL_MS` | 2000 | 等待 assistant 回复的轮询间隔 |
| `PART_PREVIEW_LEN` | 500 | open.rs 中 part 日志截断长度（与 session 的 120 可统一） |

**实施**：在 `open.rs` 顶部添加：

```rust
/// Poll interval when waiting for server to become ready.
const SERVER_POLL_INTERVAL_MS: u64 = 500;
/// Poll interval when waiting for assistant message.
const MESSAGE_POLL_INTERVAL_MS: u64 = 2000;
```

**子步骤**：
- **T4.1**：在 `open.rs` 顶部添加常量，替换 `500`、`2000`

---

### T5: event 模块错误处理（T5.1）

**问题**：`Err(_e) => break` 丢弃错误，难以排查 SSE 断开原因。

**文件**：`opencode-sdk/src/event.rs` 第 54 行

**修改**：

```rust
Err(e) => {
    debug!(error = %e, "event stream error, stopping");
    break;
}
```

**子步骤**：
- **T5.1**：将 `Err(_e) => break` 改为 `Err(e) => { debug!(error = %e, "event stream error, stopping"); break }`

---

### T6: stream_output 中 event_handle 的生命周期与 abort 逻辑（T6.1 ~ T6.2）

**问题**：`event_handle.abort()` 直接终止 task，可能导致：
- 未 flush 的输出
- 悬空 SSE 连接
- 若 `wait_for_assistant_response` 提前返回，abort 时机是否合理需确认

**文件**：`opencode-sdk/src/open.rs` 318-339 行

**现状**：先 spawn event 订阅，发消息，然后 `wait_for_assistant_response`，最后 `abort`。即：等待结束后才 abort，逻辑上合理。

**改进**：
1. 在 `abort` 后添加短延时（如 100ms），让 task 有时间退出
2. 或使用 `tokio::select!`：任一完成即取消另一个，避免显式 abort
3. 文档注释说明：`stream_output` 模式下，event 任务会在获得完整回复后被终止

**子步骤**：
- **T6.1**：在 `event_handle.abort()` 前添加注释：说明 stream_output 模式下，event 任务在获得完整回复后被终止，abort 为预期行为
- **T6.2**：（可选）`abort()` 后 `sleep(50ms).await` 再返回，给 task 退出时间

---

## 5. P2 低优先级任务

### T7: session.rs 拆分 message 解析（T7.1 ~ T7.4）

**问题**：`parse_message_list`、`parse_message_item` 与 `Client` 的 session 扩展方法混在一起，违反“一个类型一个文件”。

**方案**：新增 `opencode-sdk/src/message.rs`：

```
message.rs
├── parse_message_list(body: &str) -> Result<Vec<MessageListItem>, Error>
├── parse_message_item(v: &Value) -> Result<MessageListItem, Error>
└── log_part_received(index, part)  // 可选，若与 session 耦合可保留在 session
```

**子步骤**：
- **T7.1**：新建 `opencode-sdk/src/message.rs`，将 `parse_message_list` 迁入并 `pub use`，`session.rs` 中删除该函数
- **T7.2**：将 `parse_message_item` 迁入 `message.rs`，`session.rs` 中删除
- **T7.3**：`session.rs` 中 `session_list_messages_at` 内解析改为 `message::parse_message_list`
- **T7.4**：视情况将 `log_part_received` 迁入 `message.rs` 或保留在 `session.rs`，统一调用

---

### T8: 健康检查复用 ReqwestClient（T8.1 ~ T8.2）

**问题**：`check_server_healthy` 每次新建 `ReqwestClient`，轮询时会产生多余开销。

**文件**：`opencode-sdk/src/open.rs` 430-437 行

**子步骤**：
- **T8.1**：在 `OpenCode::open` 开头创建 `let health_client = ReqwestClient::builder().timeout(...).build().unwrap_or_else(...)`（或处理失败）
- **T8.2**：`check_server_healthy(base_url, timeout_ms)` 改为 `check_server_healthy(base_url, client: &ReqwestClient)`，删除内部 client 创建

---

### T9: ClientBuilder.build() 返回 Result（T9.1 ~ T9.3）

**问题**：`builder.build().expect("reqwest client build")` 在构建失败时 panic。

**文件**：`opencode-sdk/src/client.rs` 第 73-83 行

**子步骤**：
- **T9.1**：`error.rs` 新增 `ClientBuildFailed(#[source] reqwest::Error)` 变体
- **T9.2**：`client.rs` 中 `build() -> Client` 改为 `build() -> Result<Client, Error>`，内部 `map_err(Error::ClientBuildFailed)`
- **T9.3**：`grep -r "Client::builder\|\.build()"` 查找调用点，添加 `?` 或 `unwrap` 处理

---

### T10: 设计文档同步（T10.1 ~ T10.2）

**问题**：`open-method-design.md` 中 `ServerHandle::shutdown` 为 `async fn shutdown() -> Result<(), Error>`，实际为同步 `fn shutdown()` 且无返回值。

**文件**：`docs/rust-opencode-sdk/open-method-design.md`

**子步骤**：
- **T10.1**：`open-method-design.md` 中 `ServerHandle::shutdown` 改为 `fn shutdown(&self)`（同步、无返回值）
- **T10.2**：补充说明：best-effort 设计，进程已退出时静默忽略；可选演进为 async/Result

---

## 6. 实施顺序建议

```
Week 1（P0）:
  T1.1 -> T1.2 -> T1.3
  T2.1 -> T2.2 -> T2.3 -> T2.4 -> T2.5 -> T2.6 -> T2.7

Week 2（P1）:
  T4.1 -> T5.1
  T3.1 -> T3.2 -> T3.3 -> T3.4 -> T3.5
  T6.1 -> T6.2

Week 3（P2）:
  T10.1 -> T10.2
  T7.1 -> T7.2 -> T7.3 -> T7.4
  T8.1 -> T8.2
  T9.1 -> T9.2 -> T9.3
```

---

## 7. 验收标准

- **T1**：`cargo build` 通过，无 `static mut`，examples 正常运行
- **T2**：`cargo test` 全部通过，至少覆盖 detect、parse、extract 三类
- **T3**：`maybe_send_chat` 行数 < 80，子函数各有单一职责
- **T4**：无裸 magic number（500、2000）
- **T5**：event 流错误有 debug 日志
- **T6**：stream_output 路径有注释说明 abort 行为
- **T7**：`message.rs` 存在，session 解析逻辑迁入
- **T8**：健康检查仅创建一次 client
- **T9**：`ClientBuilder::build` 返回 `Result`，调用方处理错误
- **T10**：设计文档与实现一致

---

## 8. 风险与回退

| 任务 | 风险 | 回退 |
|------|------|------|
| T1 | 调用方未保存 guard，文件日志提前停止 | 恢复 `static mut`，或提供 `init_logger_simple()` 保留旧行为 |
| T9 | 破坏性 API 变更 | 保留 `build()`，新增 `try_build() -> Result`，或加 `#[deprecated]` 引导迁移 |

---

## 9. 附录：涉及文件清单

| 文件 | 涉及任务 |
|------|----------|
| `opencode-sdk/src/log.rs` | T1 |
| `opencode-sdk/src/open.rs` | T3, T4, T6, T8 |
| `opencode-sdk/src/event.rs` | T2, T5 |
| `opencode-sdk/src/session.rs` | T2, T7 |
| `opencode-sdk/src/client.rs` | T9 |
| `opencode-sdk/src/error.rs` | T9 |
| `opencode-sdk/tests/*.rs` | T2（新建） |
| `opencode-sdk/src/message.rs` | T7（新建） |
| `docs/rust-opencode-sdk/open-method-design.md` | T10 |

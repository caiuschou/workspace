# 架构文档与实现开发计划

> 本文档为 [architecture.md](../architecture.md) 及其子文档所描述的架构与实现，安排开发计划。任务来源于 [07-refactoring-plan](07-refactoring-plan.md)、[09-architecture-optimization](09-architecture-optimization.md)、[08-roadmap](08-roadmap.md)，完成后在「状态」列标记。**任务已拆分为细粒度子项，便于单步执行与标记。**

## 1. 文档维护与同步

| 编号 | 任务 | 关联文档 | 状态 | 优先级 |
|------|------|----------|------|--------|
| D1.1 | **代码结构变更后，更新 README 与 01** | | 完成 | P1 |
| D1.1.1 | 更新 [README](README.md) 的 Project Structure 树状结构 | README | 完成 | P1 |
| D1.1.2 | 更新 README「与 OpenCode Serve API 的对应关系」表（模块与状态列） | README | 完成 | P1 |
| D1.1.3 | 更新 [01-components](01-components.md)「SDK 与 Serve API 模块对应」表 | 01-components | 完成 | P1 |
| D1.2 | **代码结构变更后，更新 09 现状** | | 完成 | P1 |
| D1.2.1 | 更新 [09](09-architecture-optimization.md)「当前源码结构（现状）」目录树 | 09 | 完成 | P1 |
| D1.2.2 | 更新 09 现状说明（入口/核心/能力层三段） | 09 | 完成 | P1 |
| D1.3 | **opencode-serve-api 与 SDK 实现同步** | | 完成 | P2 |
| D1.3.1 | 核对 [opencode-serve-api/README](../../opencode-serve-api/README.md) 状态列与 opencode-sdk 实现 | opencode-serve-api | 完成 | P2 |
| D1.3.2 | 可选：编写脚本或 CI 步骤，从源码生成/校验状态列 | opencode-serve-api | 待办 | P2 |
| D1.4 | [01-components](01-components.md)「SDK 与 Serve API 模块对应」表与源码、Serve API 文档一致 | 01-components | 完成 | P2 |
| D1.5 | [08-roadmap](08-roadmap.md) 的 Additional APIs 表与当前实现状态一致（全部实现则更新说明） | 08-roadmap | 完成 | P2 |

## 2. 代码重构（Phase 1，高收益低风险）

| 编号 | 任务 | 关联文档 | 状态 | 优先级 |
|------|------|----------|------|--------|
| R1.1 | **新增 request.rs，统一 directory query** | 07-1.1, 09 | 完成 | P0 |
| R1.1.1 | 新建 `src/request.rs`，定义 `RequestBuilderExt` trait 与 `with_directory` 实现 | 07-1.1 | 完成 | P0 |
| R1.1.2 | 在 `lib.rs` 或需用处引入 `request` 模块（pub(crate)），event.rs 中所有请求改为 `.with_directory(directory)` | 07-1.1 | 完成 | P0 |
| R1.1.3 | file.rs 中 file_list、file_content、file_status 改为使用 `with_directory` | 07-1.1 | 完成 | P0 |
| R1.1.4 | session/mod.rs 中所有带 directory 的请求改为使用 `with_directory` | 07-1.1 | 完成 | P0 |
| R1.1.5 | 其余模块（instance, project, path_vcs, config, provider, auth, permission, question, command, find, lsp_mcp, api_log, pty, tui, experimental, agent_skill）逐一改为使用 `with_directory` | 07-1.1 | 完成 | P0 |
| R1.2 | **open.rs 内统一 run_with_stream / run_without_stream** | 07-1.2 | 完成 | P0 |
| R1.2.1 | 在 open.rs 中定义 `StreamMode` 枚举（StreamToStdout / Silent / Custom） | 07-1.2 | 完成 | P0 |
| R1.2.2 | 抽取 `run_send_and_wait(client, directory, session_id, content, wait_ms, mode)`，内含 SSE 订阅、超时、fetch_last_assistant_message | 07-1.2 | 完成 | P0 |
| R1.2.3 | `run_with_stream` 改为调用 `run_send_and_wait(..., StreamMode::StreamToStdout)` | 07-1.2 | 完成 | P0 |
| R1.2.4 | `run_without_stream` 改为调用 `run_send_and_wait(..., StreamMode::Silent)` | 07-1.2 | 完成 | P0 |
| R1.2.5 | 删除原 `run_with_stream` / `run_without_stream` 重复实现，跑通现有调用点与测试 | 07-1.2 | 完成 | P0 |
| R1.3 | **open.rs 内抽取 connect_and_maybe_chat / finish_open 类 helper** | 07-1.3 | 完成 | P0 |
| R1.3.1 | 抽取 `connect_and_maybe_chat(base_url, working_dir, options) -> Result<(Client, Option<ServerHandle>, Option<Session>, Option<MessageListItem>)>`，内部调用 Client::new、maybe_send_chat | 07-1.3 | 完成 | P0 |
| R1.3.2 | 各分支（auto_start=false / 已健康 / spawn 后）改为调用该 helper，再根据是否有 server 组装 `OpenResult { client, server, session, assistant_reply }` | 07-1.3 | 完成 | P0 |
| R1.4 | **Phase 1 完成后文档同步** | | 完成 | P1 |
| R1.4.1 | 更新 09「当前源码结构」：若已引入 request.rs，在树中体现 | 09, README | 完成 | P1 |
| R1.4.2 | 更新 README Project Structure（若有 request.rs） | README | 完成 | P1 |

## 3. 代码重构（Phase 2–4，类型与结构）

| 编号 | 任务 | 关联文档 | 状态 | 优先级 |
|------|------|----------|------|--------|
| R2.1 | **ClientBuilder::try_build() 返回 Result** | 07-2.1 | 完成 | P1 |
| R2.1.1 | 在 error.rs 中新增 `Error::ClientBuildFailed(String)` | 07-2.1 | 完成 | P1 |
| R2.1.2 | 在 client.rs 中新增 `try_build() -> Result<Client, Error>`，内部用 `map_err` 替代 `expect` | 07-2.1 | 完成 | P1 |
| R2.1.3 | 保留 `build()`，在文档中注明 panic 风险或标注 `#[deprecated]` 建议用 try_build | 07-2.1 | 完成 | P1 |
| R2.1.4 | 调用处（如 OpenCode::open 内 Client::new）改为 `builder().try_build()?` 或保持 build 按需 | 07-2.1 | 完成 | P1 |
| R2.2 | **引入 ProjectDirectory 类型（可选，breaking）** | 07-2.2, 09 | 完成 | P2 |
| R2.2.1 | 定义 `ProjectDirectory(Option<PathBuf>)`，提供 `none()`、`from_path()`、`as_path()` | 07-2.2 | 完成 | P2 |
| R2.2.2 | OpenOptions 中 project_path 改为或兼容 ProjectDirectory；OpenResult 与 API 方法签名逐步采用 | 07-2.2 | 完成 | P2 |
| R2.3 | **file_list / file_status / session_diff 具象化返回类型** | 07-2.3, 08 | 完成 | P2 |
| R2.3.1 | 定义 `FileEntry`（与 Serve API 12-file 一致），`file_list` 返回 `Result<Vec<FileEntry>, Error>` | 07-2.3 | 完成 | P2 |
| R2.3.2 | 定义 `FileStatus`，`file_status` 返回 `Result<Vec<FileStatus>, Error>` | 07-2.3 | 完成 | P2 |
| R2.3.3 | 定义 `DiffItem`，session_diff 返回 `Result<Vec<DiffItem>, Error>` | 07-2.3 | 完成 | P2 |
| R3.1 | **event 全量 JSON 日志改为 trace/debug** | 07-3.1 | 完成 | P2 |
| R3.1.1 | 将 event.rs 中每条 SSE 事件的全量 JSON 日志从 `info!` 改为 `trace!` 或 `debug!` | 07-3.1 | 完成 | P2 |
| R3.1.2 | 在文档或 README 中说明：需完整 payload 时设置 `RUST_LOG=opencode_sdk::event=trace` | 07-3.1 | 完成 | P2 |
| R3.2 | **event.rs 抽取 connect_sse** | 07-3.2 | 完成 | P2 |
| R3.2.1 | 抽取 `connect_sse(client, directory) -> Result<impl Stream<Item = Result<Event, Error>>, Error>`（URL 构建、with_directory、stream 创建） | 07-3.2 | 完成 | P2 |
| R3.2.2 | `subscribe_and_stream` 与 `subscribe_and_stream_until_done` 改为消费该 stream，去掉重复的 URL/请求构建 | 07-3.2 | 完成 | P2 |
| R3.3 | **open 内 log_assistant_reply 拆分为 log_part 等** | 07-3.3 | 完成 | P2 |
| R3.3.1 | 抽取 `log_part(part: &Part, index: usize)`，将 part 类型到日志格式的 match 收敛到一处 | 07-3.3 | 完成 | P2 |
| R3.3.2 | `log_assistant_reply` 改为遍历 parts 调用 `log_part`，去掉重复 match 与魔法常数 | 07-3.3 | 完成 | P2 |
| R4.1 | **open.rs 拆分为 open/** | 07-4.1, 09 | 完成 | P1 |
| R4.1.1 | 新建 `open/mod.rs`，保留 `OpenCode`、`open()` 主流程与 re-export（OpenOptions, OpenResult, ServerHandle） | 07-4.1 | 完成 | P1 |
| R4.1.2 | 新建 `open/options.rs`（或 options + result），迁移 OpenOptions、Default、builder 方法 | 07-4.1 | 完成 | P1 |
| R4.1.3 | 新建 `open/chat.rs`，迁移 `maybe_send_chat`、`run_send_and_wait`（及 StreamMode） | 07-4.1 | 完成 | P1 |
| R4.1.4 | 新建 `open/health.rs`，迁移 `check_server_healthy` | 07-4.1 | 完成 | P1 |
| R4.1.5 | 新建 `open/logging.rs`，迁移 `log_assistant_reply`、`log_part` | 07-4.1 | 完成 | P1 |
| R4.1.6 | 删除根目录 `open.rs` 或改为 `pub use open::*`，确保 lib.rs 与现有调用兼容 | 07-4.1 | 完成 | P1 |
| R4.2 | **session_list_messages 回退逻辑改为循环/表驱动** | 07-4.2 | 完成 | P2 |
| R4.2.1 | 将 /message、/messages、无 directory 等尝试改为 `[(path, directory); N]` 循环，首次成功即返回 | 07-4.2 | 完成 | P2 |
| R4.3 | **server 模块 CommandRunner trait（可选）** | 07-4.3 | 完成 | P3 |
| R4.3.1 | 定义 `CommandRunner` trait（如 `fn run(&self, cmd: &str, args: &[&str]) -> Result<Output>`），默认实现用 `std::process::Command` | 07-4.3 | 完成 | P3 |
| R4.3.2 | detect/install/spawn 通过参数或环境注入 CommandRunner，测试时可注入 mock | 07-4.3 | 完成 | P3 |

## 4. 颗粒度要求与架构优化

颗粒度要求见 [09-architecture-optimization](09-architecture-optimization.md)「颗粒度要求」表：**一个类型一个文件**、**单文件 ≤300 行**、**目录 mod 仅编排与 re-export**、**测试独立**。

| 编号 | 任务 | 关联文档 | 状态 | 优先级 |
|------|------|----------|------|--------|
| A4.1 | 可选：Client 与 ClientBuilder 拆为 client.rs / client_builder.rs | 09 | 完成 | P3 |
| A4.2 | **大文件按颗粒度拆目录** | 09 | 完成 | P2 |
| A4.2.1 | event.rs 拆为 event/：mod.rs（subscribe_* re-export）、connect.rs（connect_sse）、completion.rs、delta.rs | 09 | 完成 | P2 |
| A4.2.2 | file.rs 拆为 file/：mod.rs（impl Client）、types.rs（FileEntry, FileStatus，与 R2.3 协同） | 09 | 完成 | P2 |
| A4.3 | **session/ 增加 types.rs、diff.rs** | 09, R2.3 | 完成 | P2 |
| A4.3.1 | session/types.rs：Session、CreateSessionRequest 等请求/响应类型迁入或新建 | 09 | 完成 | P2 |
| A4.3.2 | session/diff.rs：DiffItem 及 session_diff 响应类型（与 R2.3.3 协同），mod 中 diff 方法可调用 | 09 | 完成 | P2 |
| A4.4 | **新增能力模块约定**：先单文件；类型≥3 或文件>300 行再拆目录，mod.rs 仅 re-export 与 impl Client | 09 | 完成 | P2 |

## 5. 路线图与配置

| 编号 | 任务 | 关联文档 | 状态 | 优先级 |
|------|------|----------|------|--------|
| M5.1 | 超时与重试抽到 Config 或 OpenOptions 扩展，减少魔法数字 | 08 | 完成 | P2 |
| M5.2 | 连接池配置暴露（高并发场景可选） | 08 | 完成 | P3 |

## 6. 执行顺序建议

1. **文档**：D1.1.x、D1.2.x 在每次较大代码结构变更后执行；D1.3–D1.5 可定期或发布前核对。
2. **重构**：R1.1.1→R1.1.5 → R1.2.1→R1.2.5 → R1.3.1→R1.3.2 → R3.1.x → R3.2.x → R4.2.1 → R2.1.x → R4.1.1→R4.1.6；R2.2、R2.3、A4.2、A4.3 视版本与 breaking 安排。
3. **颗粒度**：A4.2.x、A4.3.x 在 Phase 1 稳定后按文件行数与职责推进；A4.1、A4.4、M5.x 按需。

## 7. 颗粒度检查（发布/合入前建议）

- 新加类型是否独立成文件（或与极简类型共文件并注释说明）。
- 单文件是否超过 300 行或多职责，是否已拆为子模块/目录。
- 目录模块的 `mod.rs` 是否仅做 re-export 与 `impl Client` 编排。
- 测试是否独立（tests/ 或 `#[cfg(test)]`），是否 BDD 风格、用例有注释。

## 8. 状态约定

| 状态 | 含义 |
|------|------|
| 待办 | 未开始 |
| 进行中 | 开发中 |
| 完成 | 已完成并合并 |
| 跳过 | 暂不实施 |

完成某子任务后，将对应行的「状态」改为「完成」；父任务可在所有子任务完成后再标为「完成」，或按需在父行注明「部分完成」。

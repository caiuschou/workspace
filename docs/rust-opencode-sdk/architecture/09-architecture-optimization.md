# 代码架构的优化

> 在保持现有设计目标的前提下，从分层、边界、可扩展性与可测试性等维度指导架构演进。具体任务见 [07-refactoring-plan](07-refactoring-plan.md)。

## 当前源码结构（现状）

以下依据 **opencode-sdk 当前源码** 整理。Capability 层已覆盖 [Serve API](https://github.com/opencodecn/docs/opencode-serve-api) 全部模块，入口与核心为单文件，仅 session 为目录；`request.rs` 提供 `RequestBuilderExt::with_directory`，各 API 模块统一使用。

```
src/
├── lib.rs                 # 根 API、re-export
├── client.rs              # Client + ClientBuilder，Global(01) health、global_dispose
├── error.rs
├── log.rs                 # init_logger（SDK 侧日志）
├── open/                  # OpenCode::open 入口与编排
│   ├── mod.rs             # OpenCode、open() 主流程与 re-export
│   ├── options.rs         # OpenOptions、OpenResult、ServerHandle
│   ├── chat.rs            # connect_and_maybe_chat、run_send_and_wait、StreamMode
│   ├── health.rs          # check_server_healthy
│   └── logging.rs         # log_assistant_reply、log_part
├── request.rs             # RequestBuilderExt::with_directory（统一 directory query）
├── server/                # 进程生命周期
│   ├── mod.rs │ detect.rs │ install.rs │ spawn.rs │ shutdown.rs
├── instance.rs            # (02) POST /instance/dispose
├── project.rs             # (03) GET /project, GET /project/current, PATCH /project/{id}
├── path_vcs.rs            # (04) GET /path, GET /vcs
├── config.rs              # (05) GET/PATCH /config, GET /config/providers
├── provider.rs            # (06) GET /provider, GET /provider/auth, OAuth authorize/callback
├── auth.rs                # (07) PUT /auth/{providerID}
├── session/               # (08) 唯一能力层目录
│   ├── mod.rs             # Session 类型与全部 session_* 方法
│   └── message.rs         # Part、MessageListItem、parse_message_list 等
├── permission.rs          # (09) GET /permission, POST /permission/{id}/reply
├── question.rs            # (10) GET /question, reply, reject
├── command.rs             # (11) GET /command
├── file.rs                # (12) GET /file, GET /file/content, GET /file/status
├── find.rs                # (13) GET /find, GET /find/file, GET /find/symbol
├── lsp_mcp.rs             # (14) LSP/Formatter/MCP 状态与 MCP 增删、OAuth、connect/disconnect
├── agent_skill.rs         # (15) GET /agent, GET /skill
├── api_log.rs             # (16) POST /log（服务端写日志，与 log.rs 区分）
├── event.rs               # (17) GET /event、GET /global/event，subscribe_* 与流式解析
├── pty.rs                 # (18) PTY CRUD 与 pty_connect_url
├── tui.rs                 # (19) TUI 各类控制接口
└── experimental.rs        # (20) tool/resource/worktree 等实验接口
```

- **入口**：`open/` 目录承载 OpenCode、open() 编排；options、chat、health、logging 子模块分工。
- **核心**：`client.rs` 承载 Client、ClientBuilder、health、global_dispose；`request.rs` 提供 `RequestBuilderExt::with_directory`，各 API 模块统一使用。
- **能力层**：与 Serve API 01–20 一一对应；除 `session/` 为目录（mod + message）外，其余均为单文件 `impl Client`。

## 优化后的代码架构图

下图表示按本文原则与 [07-refactoring-plan](07-refactoring-plan.md) 实施后的目标架构：入口拆分、公共请求构造收敛、依赖单向。

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                               User Application                                    │
└─────────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│  Entry: OpenCode::open()                                                          │
│  ┌───────────────────────────────────────────────────────────────────────────┐   │
│  │  src/open/                                                                  │   │
│  │  ├── mod.rs       → OpenCode, OpenOptions, OpenResult, open() 编排          │   │
│  │  ├── options.rs   → OpenOptions + builder                                   │   │
│  │  ├── chat.rs      → connect_and_maybe_chat, run_send_and_wait               │   │
│  │  ├── health.rs    → check_server_healthy                                     │   │
│  │  └── logging.rs   → log_assistant_reply                                      │   │
│  └───────────────────────────────────────────────────────────────────────────┘   │
│         │                              │                                          │
│         │                              │  (进程生命周期，与 HTTP 无关)              │
│         ▼                              ▼                                          │
│  ┌──────────────┐            ┌─────────────────────────────────────────────────┐  │
│  │   Client     │            │  src/server/                                     │  │
│  │  (health)    │            │  detect → install → spawn → shutdown              │  │
│  └──────────────┘            └─────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│  Core: Client (client.rs)                                                        │
│  ┌───────────────────────────────────────────────────────────────────────────┐   │
│  │  base_url, http (reqwest), ClientBuilder → build() / try_build()           │   │
│  └───────────────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────────────┐     │
│  │  Shared: src/request.rs → RequestBuilderExt::with_directory(directory)  │     │
│  │  (optional) ProjectDirectory 类型统一 “项目目录” 语义                      │     │
│  └─────────────────────────────────────────────────────────────────────────┘     │
│                                                                                   │
│  Capability 与 [Serve API](../../opencode-serve-api.md) 一一对应：                   │
│  现状：全部模块已实现（单文件为主，仅 session/ 为目录）                              │
│  优化目标：request 抽取、open 拆子模块、大文件按类型/职责再拆为目录                   │
└─────────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        OpenCode Serve API（01-global … 20-experimental，见 opencode-serve-api）                           │
└─────────────────────────────────────────────────────────────────────────────────┘
```

**图例说明：**

- **Entry**：`open()` 只做编排；具体逻辑在 `open/` 子模块（chat / health / logging），与 `server/` 并列，依赖 `Client` 做健康检查。
- **Core**：`Client` 为唯一核心；`request.rs` 提供公共的 `with_directory`，被各 API 模块使用；可选 `ProjectDirectory` 统一“项目目录”语义。
- **Capability**：与 [OpenCode Serve API](../../opencode-serve-api.md) 模块一一对应；**现状**全部已实现（单文件为主，仅 session/ 为目录）；**优化目标**为抽取 request、拆分 open、大文件按类型/职责拆成目录（如 file/、event/、lsp_mcp/ 等）。
- **依赖方向**：User → open → (Client + server)；API 模块 → Client → OpenCode Serve API。

**颗粒度要求（与 [AGENTS.md](../../../AGENTS.md) Rust 约定一致）：**

| 要求 | 说明 | 阈值/例外 |
|------|------|-----------|
| **一个类型一个文件** | 每个主要类型（含请求/响应 DTO、Option/Result 包装的领域类型）独立文件，便于定位与测试。 | 仅当类型极简（如仅 1–2 个字段且无行为）可与同模块其它类型共文件，需在文件头注释说明。 |
| **单文件行数** | 单文件职责单一，避免单文件承担多类职责。 | 建议单文件 ≤ 300 行；超过时按类型或子职责拆为多文件或子模块。 |
| **目录与 mod** | 能力层（与 Serve API 对应）按模块建目录时，目录内按类型/职责分文件。 | `mod.rs` 仅做 re-export 与 `impl Client` 编排，不承载大段业务逻辑。 |
| **测试独立** | 单元测试拆分到单独目录（如 `tests/`）或同模块内 `#[cfg(test)] mod tests`。 | 遵循 AGENTS.md：测试独立、BDD 风格、用例方法有注释。 |
| **新模块准入** | 新增能力模块（新 Serve API 对应）时，先单文件 `xxx.rs`；文件膨胀或类型增多再拆目录。 | 单文件内类型 ≥3 或文件 >300 行时，建议拆为 `xxx/mod.rs` + `xxx/types.rs` 等。 |

**目标结构示意（细颗粒度，与 [Serve API 模块](../../opencode-serve-api.md) 对应）：**

下图为在现状基础上的**优化目标**；当前实现见上文「当前源码结构（现状）」。

```
src/
├── lib.rs │ client.rs │ client_builder.rs │ error.rs │ request.rs │ log.rs
├── open/                  # 入口编排（由 open.rs 拆出）
│   ├── mod.rs │ options.rs │ result.rs │ chat.rs │ health.rs │ logging.rs
├── server/
│   ├── mod.rs │ detect.rs │ install.rs │ spawn.rs │ shutdown.rs
├── instance.rs            # (02) 保持单文件或按需拆 types
├── project/               # (03) 由 project.rs 拆出目录，可选 directory.rs
├── path_vcs.rs            # (04) 保持或拆 path.rs / vcs.rs
├── config.rs              # (05)
├── provider.rs │ auth.rs  # (06)(07)
├── session/               # (08) 已为目录；可增 types.rs、diff.rs
│   ├── mod.rs │ types.rs │ message.rs │ diff.rs
├── permission.rs │ question.rs │ command.rs  # (09)(10)(11)
├── file/                  # (12) 由 file.rs 拆出 mod + types
├── find.rs                # (13) 保持或拆 find/
├── lsp_mcp/               # (14) 由 lsp_mcp.rs 拆出（文件大时可拆）
├── agent_skill.rs │ api_log.rs  # (15)(16)
├── event/                 # (17) 由 event.rs 拆出 connect/completion/delta
├── pty.rs │ tui.rs        # (18)(19)
└── experimental/          # (20) 由 experimental.rs 拆出目录（按需）
```

**与现状的差异（后续优化方向）：**

| 项 | 现状 | 优化目标 |
|----|------|----------|
| 入口 | `open.rs` 单文件 | 拆为 `open/`（mod、options、chat、health、logging） |
| 公共请求 | 各模块重复拼接 `directory` | 新增 `request.rs`，`RequestBuilderExt::with_directory` |
| Client | `client.rs` 含 Client + ClientBuilder | 可选拆出 `client_builder.rs` |
| 能力层 | 多为单文件（仅 session/ 为目录） | 大文件或超 200 行按类型/职责拆成目录（file/、event/、lsp_mcp/ 等） |
| session/ | mod + message | 可增 types.rs、diff.rs，具象化 diff 返回类型 |
| file / event | file.rs、event.rs | 拆为 file/、event/，mod + types 或 connect/completion/delta |

## 优化原则

| 原则 | 说明 |
|------|------|
| **单一职责** | 每个模块只负责一类能力（如 session 只做会话 API，server 只做进程生命周期）。 |
| **颗粒度** | 一个类型一个文件；单文件建议 ≤300 行；超则按类型/子职责再拆；目录下 `mod.rs` 仅编排与 re-export。详见本文「颗粒度要求」表。 |
| **依赖单向** | 上层依赖下层：`open` → `client` + `server`；API 模块只依赖 `Client`，不互相依赖。 |
| **接口稳定** | 公开 API（`Client` 方法、`OpenOptions`/`OpenResult`）变更需谨慎，内部实现可逐步重构。 |
| **简单优先** | 不引入不必要的抽象；新类型、新模块在确有重复或边界清晰时再抽取。 |

## 分层与边界

### 当前分层（目标保持）

```
User / open()  →  Client + server  →  OpenCode Server (HTTP/SSE)
```

- **入口层**：`OpenCode::open()`、`Client::new` / `ClientBuilder`。只做编排与配置，不承载业务细节。
- **核心层**：`Client` 持有 HTTP 与 base_url；各 API 通过 `impl Client` 扩展，不修改 `client.rs` 核心。
- **能力层**：与 [OpenCode Serve API](../../opencode-serve-api.md) 模块一一对应；**现状**为单文件为主（instance.rs、project.rs、…、experimental.rs），仅 `session/` 为目录；`server/` 封装 detect/install/spawn/shutdown，与 HTTP 无关。

### 边界优化方向

1. **`open.rs` 瘦身**  
   将“连接 + 可选聊天”与“健康检查 / 启动流程”拆成小函数或子模块（如 `open/chat.rs`、`open/health.rs`），避免单文件承担过多职责。参见 [07-refactoring-plan](07-refactoring-plan.md) 4.1。

2. **`directory` 语义统一**  
   多处 `Option<&Path>` 易混淆“未传”与“传了空”。可引入 `ProjectDirectory` 等类型统一表示“项目目录上下文”，减少传参错误。参见 07 中 2.2。

3. **HTTP 请求构造收敛**  
   **现状**：各模块（event、file、session、instance、project 等）内重复写 `if let Some(dir) = directory { req = req.query(&[("directory", s)]); }`。**优化**：新增 `request.rs`，`RequestBuilderExt::with_directory` 统一处理，避免重复。参见 07 中 1.1。

## 可扩展性优化

- **新 API 以 `impl Client` 形式扩展**  
  新端点放在独立模块（如 `project.rs`），在 `lib.rs` 中 re-export；不修改 `client.rs` 或其它 API 模块。参见 [06-extension-guide](06-extension-guide.md)。

- **类型与模块一一对应**  
  新请求/响应类型尽量单独文件或与对应 API 同模块，便于维护和测试。例如 `session_diff` 的返回类型可放在 `session/diff.rs` 或 `session/mod.rs`。

- **配置与常量集中**  
  超时、重试等从魔法数字抽到 `OpenOptions` 或未来 `Config`，便于调优和文档化。参见 [08-roadmap](08-roadmap.md) Configuration Abstraction。

## 可测试性优化

- **pub(crate) 边界**  
  仅内部使用的函数/类型使用 `pub(crate)`，便于在 crate 内单测而不暴露到公开 API。

- **依赖可替换（中长期）**  
  `server/` 中 detect、install、spawn 若需集成测试或 mock，可引入 trait（如 `CommandRunner`）再提供默认实现与测试实现。参见 07 中 4.3。

- **测试风格**  
  单元测试与集成测试保持 BDD 风格、用例名即文档；详见 [04-testing](04-testing.md)。

## 与重构计划的关系

- **07-refactoring-plan** 按阶段列出具体任务（抽取 helper、统一 stream、类型具象化等），是“做什么”。
- **本文档** 从架构原则和边界出发，说明“为什么这样拆、怎样保持清晰”，是“怎么想”；并基于**当前源码**补充了「当前源码结构（现状）」与「与现状的差异」，便于从现状推进到目标结构。

实施时优先做 07 中 Phase 1 的高收益低风险项（如 request 抽取、open 内 run_send_and_wait 统一），再结合本文的分层与边界原则逐步推进 Phase 2–4；能力层在保持与 Serve API 一一对应的前提下，按文件体积与职责再拆目录（如 file/、event/），避免在单点堆积过多职责。

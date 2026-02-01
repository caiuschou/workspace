# OpenCode SDK Architecture

> Rust SDK for [OpenCode Server](https://opencodecn.com/docs/server) API — Type-safe, async-first, batteries-included.

## Overview

OpenCode SDK is a Rust client library that provides a seamless interface to the **OpenCode Serve API**（接口按模块拆分，见 [opencode-serve-api 文档](../../opencode-serve-api.md)）。It enables developers to programmatically interact with AI coding assistants, manage sessions, and handle streaming responses.

### Design Goals

| Goal | Description |
|------|-------------|
| **Type Safety** | Leverage Rust's type system to catch errors at compile time. All API requests and responses use strongly-typed definitions with serde serialization. |
| **Async-First** | Built on `tokio` and `reqwest` for non-blocking I/O. All network operations are async by default. |
| **Zero Configuration** | Works out of the box with sensible defaults. Auto-detects, auto-installs, and auto-starts the server when needed. |
| **Extensible** | API methods organized as `impl Client` extensions, making it easy to add new endpoints without breaking changes. |
| **Production Ready** | Comprehensive error handling, structured logging, and graceful shutdown support. |

### Non-Goals

- **GUI/TUI**: This is a library, not an application. Use it to build your own tools.
- **Multi-server**: Designed for single-server scenarios. For multi-server, create multiple `Client` instances.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              User Application                                │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                            OpenCode::open()                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Detect    │→ │   Install   │→ │    Spawn    │→ │   Health Check      │ │
│  │  (server/)  │  │  (server/)  │  │  (server/)  │  │   (client.rs)       │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                               Client                                         │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  Base: HTTP client (reqwest), base_url, timeout configuration        │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│  Capability 与 [Serve API 模块](../../opencode-serve-api.md) 一一对应：       │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐│
│  │ Global   │ │ Session  │ │ File     │ │ Event    │ │ 其余模块           ││
│  │(01) 部分 │ │(08) 部分 │ │(12) 部分 │ │(17) 完成 │ │(02–07,09–16,18–20)││
│  │ health   │ │ create,  │ │ list,    │ │ SSE 订阅 │ │ 未实现，见 roadmap ││
│  │          │ │ msg,diff │ │ status   │ │          │ │                    ││
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│              OpenCode Serve API（按模块：Global/Instance/Project/…/Event）   │
│                        HTTP REST + SSE Events                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
opencode-sdk/
├── Cargo.toml              # Package manifest with dependencies
├── README.md               # Quick start guide
├── src/
│   ├── lib.rs              # Public API exports
│   ├── client.rs           # HTTP Client, health check, ClientBuilder
│   ├── error.rs            # Error types (thiserror-based)
│   ├── event.rs            # SSE event streaming and parsing
│   ├── file.rs             # File/directory API (impl Client)
│   ├── log.rs              # Logging initialization (tracing)
│   ├── open/               # OpenCode::open one-shot entry point
│   │   ├── mod.rs          # OpenCode, open() orchestration, re-exports
│   │   ├── options.rs      # OpenOptions, OpenResult, ServerHandle
│   │   ├── chat.rs         # connect_and_maybe_chat, run_send_and_wait
│   │   ├── health.rs       # check_server_healthy
│   │   └── logging.rs      # log_assistant_reply, log_part
│   ├── request.rs          # RequestBuilderExt::with_directory (shared)
│   ├── server/             # Server process management
│   │   ├── mod.rs          # Module re-exports
│   │   ├── detect.rs       # Command detection (which/where)
│   │   ├── install.rs      # Auto-installation (npm/brew/curl)
│   │   ├── spawn.rs        # Process spawning
│   │   └── shutdown.rs     # Graceful termination
│   └── session/            # Session API
│       ├── mod.rs          # Session, Part, request/response types
│       └── message.rs      # Message parsing utilities
├── examples/
│   ├── open.rs             # Connect to existing server
│   ├── cli_fibonacci.rs    # Full example with project path + chat
│   └── parse_log.rs        # Log parsing utility
└── tests/
    ├── common.rs           # Test utilities
    └── detect_test.rs      # Command detection tests
```

## 与 OpenCode Serve API 的对应关系

SDK 的 Capability 层与 [OpenCode Serve API](../../opencode-serve-api.md) 的模块一一对应；接口列表与状态见 [opencode-serve-api/README.md](../../opencode-serve-api/README.md)。

| Serve API 模块 | 文档 | SDK 对应 | 状态 |
|----------------|------|----------|------|
| Global 全局 | [01-global](../../opencode-serve-api/01-global.md) | `Client::health()`、`global_dispose` | 已实现 |
| Instance 实例 | [02-instance](../../opencode-serve-api/02-instance.md) | `instance.rs` | 已实现 |
| Project 项目 | [03-project](../../opencode-serve-api/03-project.md) | `project.rs` | 已实现 |
| Path & VCS | [04-path-vcs](../../opencode-serve-api/04-path-vcs.md) | `path_vcs.rs` | 已实现 |
| Config 配置 | [05-config](../../opencode-serve-api/05-config.md) | `config.rs` | 已实现 |
| Provider / Auth | [06-provider](../../opencode-serve-api/06-provider.md), [07-auth](../../opencode-serve-api/07-auth.md) | `provider.rs`、`auth.rs` | 已实现 |
| Session / Message | [08-session](../../opencode-serve-api/08-session.md) | `session/` | 已实现 |
| Permission / Question / Command | 09–11 | `permission.rs`、`question.rs`、`command.rs` | 已实现 |
| File 文件 | [12-file](../../opencode-serve-api/12-file.md) | `file.rs` | 已实现 |
| Find / LSP / MCP / Agent / Logging | 13–16 | `find.rs`、`lsp_mcp.rs`、`agent_skill.rs`、`api_log.rs` | 已实现 |
| Event 事件 | [17-event](../../opencode-serve-api/17-event.md) | `event.rs` | 已实现 |
| PTY / TUI / Experimental | 18–20 | `pty.rs`、`tui.rs`、`experimental.rs` | 已实现 |

新增 API 时：在对应模块下按 [06-extension-guide](06-extension-guide.md) 扩展，类型与 Serve API 文档对齐。

## 文档索引

| 文档 | 描述 |
|------|------|
| [01-components](01-components.md) | Client、OpenCode::open、Server、Session、Event、File、Error、Logging |
| [02-design-patterns](02-design-patterns.md) | 设计模式与并发模型 |
| [03-security-performance](03-security-performance.md) | 安全与性能考量 |
| [04-testing](04-testing.md) | 测试策略 |
| [05-dependencies](05-dependencies.md) | 依赖说明 |
| [06-extension-guide](06-extension-guide.md) | 扩展新 API 指南 |
| [07-refactoring-plan](07-refactoring-plan.md) | 重构计划 |
| [08-roadmap](08-roadmap.md) | 路线图与参考 |
| [09-architecture-optimization](09-architecture-optimization.md) | 代码架构的优化（含颗粒度要求） |
| [10-development-plan](10-development-plan.md) | 架构文档与实现开发计划 |

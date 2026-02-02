# Roadmap

## Type Refinement

Concrete return types have been introduced (see [10-development-plan](10-development-plan.md) R2.3):

- [x] `file_list` → `Result<Vec<FileEntry>, Error>` (FileEntry in file.rs)
- [x] `file_status` → `Result<Vec<FileStatus>, Error>` (FileStatus in file.rs)
- [x] `session_diff` → `Result<Vec<DiffItem>, Error>` (DiffItem in session/diff.rs)

## Additional APIs

与 [OpenCode Serve API](../../opencode-serve-api.md) 模块对应；完整接口列表与状态见 [opencode-serve-api/README.md](../../opencode-serve-api/README.md)。**当前 SDK 已实现 01–20 全部模块对应接口**（见 [01-components](01-components.md)「SDK 与 Serve API 模块对应」表）。

| Serve API 模块 | 文档 | 说明 |
|----------------|------|------|
| Global | [01-global](../../opencode-serve-api/01-global.md) | health、global_dispose、subscribe_global_events 已实现 |
| Instance | [02-instance](../../opencode-serve-api/02-instance.md) | instance_dispose 已实现 |
| Project | [03-project](../../opencode-serve-api/03-project.md) | 项目列表、当前项目、更新 已实现 |
| Path & VCS | [04-path-vcs](../../opencode-serve-api/04-path-vcs.md) | path、vcs 已实现 |
| Config | [05-config](../../opencode-serve-api/05-config.md) | 配置读写、providers 已实现 |
| Provider / Auth | [06-provider](../../opencode-serve-api/06-provider.md), [07-auth](../../opencode-serve-api/07-auth.md) | 模型提供商、OAuth、认证 已实现 |
| Session / Message | [08-session](../../opencode-serve-api/08-session.md) | 创建/列表/消息/发送/diff/fork/abort 等 已实现 |
| Permission / Question / Command | 09–11 | 权限、问题、命令 已实现 |
| File | [12-file](../../opencode-serve-api/12-file.md) | file_list、file_content、file_status 已实现 |
| Find | [13-find](../../opencode-serve-api/13-find.md) | 文本/文件/符号搜索 已实现 |
| LSP / Formatter / MCP | [14-lsp-formatter-mcp](../../opencode-serve-api/14-lsp-formatter-mcp.md) | LSP、Formatter、MCP 已实现 |
| Agent & Skill | [15-agent-skill](../../opencode-serve-api/15-agent-skill.md) | agent_list、skill_list 已实现 |
| Logging / Event / PTY / TUI / Experimental | 16–20 | 服务端日志、Event、PTY、TUI、实验接口 已实现 |

## Configuration Abstraction

Extract hardcoded timeouts and retry logic into `Config`:

```rust
pub struct Config {
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub retry_policy: RetryPolicy,
    // ...
}
```

## Connection Pooling

Connection pool configuration is exposed on `ClientBuilder` for high-throughput scenarios:

- **`pool_max_idle_per_host(max: usize)`** — maximum idle connections per host (reqwest default: no limit).
- **`pool_idle_timeout(timeout: Option<Duration>)`** — how long idle sockets are kept (reqwest default: 90s).

Example:

```rust
use opencode_sdk::Client;
use std::time::Duration;

let client = Client::builder("http://127.0.0.1:4096")
    .pool_max_idle_per_host(20)
    .pool_idle_timeout(Some(Duration::from_secs(60)))
    .try_build()?;
```

---

## References

- [OpenCode Server API Documentation](https://opencodecn.com/docs/server)
- [Event Format Specification](../../opencode-serve-api/17-event-format.md)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

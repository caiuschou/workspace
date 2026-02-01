# Roadmap

## Type Refinement

Current `serde_json::Value` returns should be replaced with concrete types:

- [ ] `file_list` → `Vec<FileEntry>`
- [ ] `file_status` → `Vec<FileStatus>`
- [ ] `session_diff` → `Vec<DiffItem>`

## Additional APIs

与 [OpenCode Serve API](../../opencode-serve-api.md) 模块对应；完整接口列表见 [opencode-serve-api/README.md](../../opencode-serve-api/README.md)。

| Serve API 模块 | 文档 | 说明 |
|----------------|------|------|
| Global | [01-global](../../opencode-serve-api/01-global.md) | 已实现 health；未实现 event、dispose |
| Instance | [02-instance](../../opencode-serve-api/02-instance.md) | dispose |
| Project | [03-project](../../opencode-serve-api/03-project.md) | 项目列表、当前项目、更新 |
| Path & VCS | [04-path-vcs](../../opencode-serve-api/04-path-vcs.md) | path、vcs |
| Config | [05-config](../../opencode-serve-api/05-config.md) | 配置读写、providers |
| Provider / Auth | [06-provider](../../opencode-serve-api/06-provider.md), [07-auth](../../opencode-serve-api/07-auth.md) | 模型提供商、OAuth、认证 |
| Session / Message | [08-session](../../opencode-serve-api/08-session.md) | 已部分实现；未实现 list/delete/fork/abort 等 |
| Permission / Question / Command | 09–11 | 权限、问题、命令 |
| File | [12-file](../../opencode-serve-api/12-file.md) | 已实现 list、status；未实现 content |
| Find | [13-find](../../opencode-serve-api/13-find.md) | 文本/文件/符号搜索 |
| LSP / Formatter / MCP | [14-lsp-formatter-mcp](../../opencode-serve-api/14-lsp-formatter-mcp.md) | LSP、Formatter、MCP |
| Agent & Skill | [15-agent-skill](../../opencode-serve-api/15-agent-skill.md) | 代理与技能 |
| Logging / Event / PTY / TUI / Experimental | 16–20 | 服务端日志、Event 已实现，其余未实现 |

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

Consider exposing connection pool configuration for high-throughput scenarios.

---

## References

- [OpenCode Server API Documentation](https://opencodecn.com/docs/server)
- [Event Format Specification](../../opencode-serve-api/17-event-format.md)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

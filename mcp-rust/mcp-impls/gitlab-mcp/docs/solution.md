# GitLab MCP Server 解决方案

## 概述

基于 `mcp-rust` 框架创建一个 GitLab MCP (Model Context Protocol) Server，使 AI 助手能够通过 MCP 协议与 GitLab 进行交互，实现项目管理、Issue 跟踪、Merge Request 操作、CI/CD 管道管理等核心功能。

## 工具实现状态

各工具的详细实现状态请参考 [README.md](../README.md#工具列表)。

## 技术栈

- **语言**: Rust
- **MCP 框架**: [mcp-rust](../mcp-rust) (core + server + client)
- **GitLab API**: reqwest (HTTP 客户端)
- **异步运行时**: Tokio
- **序列化**: serde / serde_json
- **CLI 框架**: clap
- **输出格式化**: tabled, console
- **配置**: toml, dirs
- **URL 编码**: urlencoding

## 项目结构 (Workspace)

```
gitlab-mcp/
├── Cargo.toml                 # Workspace 配置
├── Cargo.lock
├── README.md
├── docs/
│   └── solution.md            # 本文档
│
├── crates/
│   │
│   ├── mcp-server/            # MCP Server (AI 助手调用)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs        # stdio 入口，MCP 协议处理
│   │       ├── lib.rs         # 库入口，导出核心功能
│   │       ├── server.rs      # MCP 服务器配置
│   │       ├── config.rs      # 配置管理
│   │       ├── gitlab.rs      # GitLab API 客户端
│   │       ├── error.rs       # 错误类型定义
│   │       └── tools/         # MCP 工具实现
│   │       ├── mod.rs         # 工具注册
│   │       ├── config.rs      # 配置相关工具
│   │       ├── project.rs     # 项目管理工具
│   │       ├── issue.rs       # Issue 工具
│   │       ├── merge_request.rs # MR 工具
│   │       ├── pipeline.rs    # Pipeline 工具
│   │       ├── repository.rs  # 仓库文件工具
│   │       ├── branch.rs      # 分支工具
│   │       └── commit.rs      # 提交工具
│   │
│   └── mcp-client/            # MCP Client (CLI 工具)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs        # CLI 入口
│           ├── cli.rs         # 命令行参数解析
│           ├── client.rs      # MCP 客户端，调用 mcp-server
│           ├── config.rs      # 客户端配置
│           ├── transport.rs   # MCP stdio 传输层
│           └── commands/      # 各工具的子命令
│               ├── mod.rs
│               ├── config.rs  # 配置命令
│               ├── project.rs # 项目命令
│               ├── issue.rs   # Issue 命令
│               ├── merge_request.rs # MR 命令
│               ├── pipeline.rs # Pipeline 命令
│               ├── repository.rs # 仓库命令
│               └── branch.rs  # 分支命令
│
└── tests/
    └── integration_test.rs    # 集成测试 (待实现)
```

### Workspace 配置

详见项目根目录 `Cargo.toml`。

## MCP Client 设计

### 概述

`mcp-client` 是一个 CLI 工具，为所有 MCP 工具提供子命令接口。它通过 MCP 协议调用 `mcp-server` 进行验证和执行，确保：
1. 所有操作都通过 MCP Server 验证
2. CLI 和 AI 助手使用相同的 API
3. 便于手动测试和调试

### 架构设计

```
┌─────────────┐     MCP Protocol (stdio)     ┌─────────────┐
│  CLI Client │ ─────────────────────────────▶ │ MCP Server  │
│             │                                │             │
│  - clap     │       call_tool()              │  - Tools    │
│  - mcp_sdk  │ ─────────────────────────────▶ │  - GitLab   │
└─────────────┘                                └─────────────┘
```

### 通信协议

Client 与 Server 之间通过 MCP 协议（stdio）进行 JSON-RPC 通信。

## MCP 资源设计 (Resources)

资源用于提供对 GitLab 数据的只读访问：

| 资源 URI | 描述 |
|---------|------|
| `gitlab://project/{project_id}` | 项目详细信息 |
| `gitlab://project/{project_id}/issues` | 项目 Issues 列表 |
| `gitlab://project/{project_id}/merge_requests` | 项目 MRs 列表 |
| `gitlab://project/{project_id}/pipelines` | 项目 Pipelines 列表 |
| `gitlab://project/{project_id}/repository/tree` | 仓库文件树 |
| `gitlab://user/current` | 当前用户信息 |
| `gitlab://project/{project_id}/wiki` | 项目 Wiki 页面列表 |
| `gitlab://project/{project_id}/wiki/{slug}` | 单个 Wiki 页面内容 |
| `gitlab://project/{project_id}/branches` | 项目分支列表 |
| `gitlab://project/{project_id}/commits` | 项目提交列表 |
| `gitlab://project/{project_id}/tags` | 项目标签列表 |
| `gitlab://project/{project_id}/milestones` | 项目里程碑列表 |
| `gitlab://project/{project_id}/environments` | 项目环境列表 |
| `gitlab://project/{project_id}/releases` | 项目发布列表 |

## 依赖项

项目使用的主要依赖：
- **MCP 框架**: mcp_core, mcp_server, mcp_client
- **异步运行时**: tokio, async-trait
- **序列化**: serde, serde_json
- **HTTP 客户端**: reqwest
- **URL 处理**: url, urlencoding
- **CLI**: clap
- **输出格式化**: tabled, console
- **错误处理**: anyhow, thiserror
- **日志**: tracing, tracing-subscriber
- **配置**: toml, dirs

## 错误处理策略

1. **API 错误**: 将 GitLab API 错误转换为 MCP 工具错误响应
2. **认证错误**: 清晰提示 token 无效或缺失
3. **网络错误**: 提供重试建议
4. **参数验证**: 在调用 API 前验证必需参数

## 安全考虑

1. Token 不应在日志中输出
2. 支持 Token 轮换机制
3. 实现 Rate Limiting 遵守 GitLab API 限制
4. 敏感操作 (合并 MR) 需要明确参数确认

## 参考

- [mcp-rust](../mcp-rust) - MCP Rust 实现
- [GitLab API Documentation](https://docs.gitlab.com/ee/api/api_resources.html)
- [Model Context Protocol](https://modelcontextprotocol.io/)

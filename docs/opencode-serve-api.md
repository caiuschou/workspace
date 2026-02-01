# OpenCode Serve API 文档

> **接口已按模块拆分**，请使用下方链接查找。

完整目录与接口列表：[opencode-serve-api/README.md](opencode-serve-api/README.md)

## 模块索引

> **状态**：基于 [opencode-sdk](../opencode-sdk) 的实现情况。完成 = 主要接口已实现；部分 = 部分接口已实现；未完成 = 暂未实现。

| 模块 | 文档 | 状态 |
|------|------|------|
| Global 全局 | [01-global.md](opencode-serve-api/01-global.md) | 部分 |
| Instance 实例 | [02-instance.md](opencode-serve-api/02-instance.md) | 未完成 |
| Project 项目 | [03-project.md](opencode-serve-api/03-project.md) | 未完成 |
| Path & VCS | [04-path-vcs.md](opencode-serve-api/04-path-vcs.md) | 未完成 |
| Config 配置 | [05-config.md](opencode-serve-api/05-config.md) | 未完成 |
| Provider 模型提供商 | [06-provider.md](opencode-serve-api/06-provider.md) | 未完成 |
| Auth 认证 | [07-auth.md](opencode-serve-api/07-auth.md) | 未完成 |
| Session 会话 / Message 消息 | [08-session.md](opencode-serve-api/08-session.md) | 部分 |
| Permission 权限 | [09-permission.md](opencode-serve-api/09-permission.md) | 未完成 |
| Question 问题 | [10-question.md](opencode-serve-api/10-question.md) | 未完成 |
| Command 命令 | [11-command.md](opencode-serve-api/11-command.md) | 未完成 |
| File 文件 | [12-file.md](opencode-serve-api/12-file.md) | 部分 |
| Find 查找 | [13-find.md](opencode-serve-api/13-find.md) | 未完成 |
| LSP / Formatter / MCP | [14-lsp-formatter-mcp.md](opencode-serve-api/14-lsp-formatter-mcp.md) | 未完成 |
| Agent & Skill | [15-agent-skill.md](opencode-serve-api/15-agent-skill.md) | 未完成 |
| Logging 日志 | [16-logging.md](opencode-serve-api/16-logging.md) | 未完成 |
| Event 事件 | [17-event.md](opencode-serve-api/17-event.md) | 完成 |
| PTY 伪终端 | [18-pty.md](opencode-serve-api/18-pty.md) | 未完成 |
| TUI 界面控制 | [19-tui.md](opencode-serve-api/19-tui.md) | 未完成 |
| Experimental 实验性 | [20-experimental.md](opencode-serve-api/20-experimental.md) | 未完成 |

## 主题汇总

| 主题 | 文档 |
|------|------|
| 实时接口（SSE、WebSocket、流式） | [21-realtime.md](opencode-serve-api/21-realtime.md) |

## 重新生成

接口文档由脚本从 OpenAPI 生成：

```bash
python scripts/gen-opencode-api-docs.py [URL或openapi.json路径]
```

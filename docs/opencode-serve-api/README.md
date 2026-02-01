# OpenCode Serve API 文档

按模块拆分的接口文档，便于按功能查找。

## 通用说明

- **directory 参数**：多数接口支持 query 参数 `directory`，用于指定工作目录；未传时使用当前实例的 cwd。若某接口的 `directory` 说明为空，则遵循此约定。
- 每个接口文档包含：OperationId、摘要、说明、请求参数表、请求体表（如有）、响应表。

## 重新生成

```bash
python scripts/gen-opencode-api-docs.py [URL或openapi.json路径]
```

默认从 `http://host.docker.internal:34917/doc` 拉取 OpenAPI。

---

## 按主题

| 主题 | 说明 |
|------|------|
| [实时接口](21-realtime.md) | SSE 事件流、WebSocket、流式 AI 响应等实时能力汇总 |

---

## 目录（按模块）

表格四列：**类别**（链接到该模块文档）、**接口**（METHOD 路径）、**描述**（接口用途）、**状态**（[opencode-sdk](../opencode-sdk) 是否已实现）。**状态依据 opencode-sdk 源码核对。**

| 类别 | 接口 | 描述 | 状态 |
|------|------|------|------|
| [Global 全局](01-global.md) | `GET /global/health` | 获取服务健康与版本 | 已实现 |
| [Global 全局](01-global.md) | `GET /global/event` | 订阅全局事件流（SSE） | 已实现 |
| [Global 全局](01-global.md) | `POST /global/dispose` | 销毁所有 OpenCode 实例 | 已实现 |
| [Instance 实例](02-instance.md) | `POST /instance/dispose` | 销毁当前实例，释放资源 | 已实现 |
| [Project 项目](03-project.md) | `GET /project` | 列出已打开的项目 | 已实现 |
| [Project 项目](03-project.md) | `GET /project/current` | 获取当前活动项目 | 已实现 |
| [Project 项目](03-project.md) | `PATCH /project/{projectID}` | 更新项目（name、icon、commands） | 已实现 |
| [Path & VCS](04-path-vcs.md) | `GET /path` | 获取当前工作目录与路径信息 | 已实现 |
| [Path & VCS](04-path-vcs.md) | `GET /vcs` | 获取版本控制信息（如 git 分支） | 已实现 |
| [Config 配置](05-config.md) | `GET /config` | 获取当前配置 | 已实现 |
| [Config 配置](05-config.md) | `PATCH /config` | 更新配置 | 已实现 |
| [Config 配置](05-config.md) | `GET /config/providers` | 列出 providers 与默认模型 | 已实现 |
| [Provider 模型提供商](06-provider.md) | `GET /provider` | 列出所有可用/已连接提供商 | 已实现 |
| [Provider 模型提供商](06-provider.md) | `GET /provider/auth` | 获取各提供商认证方式 | 已实现 |
| [Provider 模型提供商](06-provider.md) | `POST /provider/{providerID}/oauth/authorize` | 发起 OAuth 授权，获取授权 URL | 已实现 |
| [Provider 模型提供商](06-provider.md) | `POST /provider/{providerID}/oauth/callback` | 处理 OAuth 回调 | 已实现 |
| [Auth 认证](07-auth.md) | `PUT /auth/{providerID}` | 设置某提供商的认证凭据 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `GET /session` | 列出会话（可按目录、标题、时间等筛选） | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `POST /session` | 创建新会话 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `GET /session/status` | 获取所有会话状态（active/idle/completed） | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `GET /session/{sessionID}` | 获取会话详情 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `DELETE /session/{sessionID}` | 删除会话及全部数据 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `PATCH /session/{sessionID}` | 更新会话（如 title） | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `GET /session/{sessionID}/children` | 获取从该会话 fork 出的子会话 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `GET /session/{sessionID}/todo` | 获取会话待办列表 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `POST /session/{sessionID}/init` | 初始化会话（分析项目并生成 AGENTS.md） | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `POST /session/{sessionID}/fork` | 在指定消息处 fork 出新会话 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `POST /session/{sessionID}/abort` | 中止正在运行的会话 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `POST /session/{sessionID}/share` | 创建可分享链接 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `DELETE /session/{sessionID}/share` | 取消分享 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `GET /session/{sessionID}/diff` | 获取某条消息导致的文件变更 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `POST /session/{sessionID}/summarize` | 用 AI 总结会话 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `GET /session/{sessionID}/message` | 获取会话内消息列表 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `POST /session/{sessionID}/message` | 发送消息并流式返回 AI 响应 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `POST /session/{sessionID}/prompt_async` | 异步发送消息，立即返回 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `GET /session/{sessionID}/message/{messageID}` | 获取单条消息详情 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `DELETE /session/{sessionID}/message/{messageID}/part/{partID}` | 删除消息中的某 part | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `PATCH /session/{sessionID}/message/{messageID}/part/{partID}` | 更新消息中的某 part | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `POST /session/{sessionID}/revert` | 回滚某条消息 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `POST /session/{sessionID}/unrevert` | 恢复所有已回滚消息 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `POST /session/{sessionID}/permissions/{permissionID}` | 批准或拒绝权限请求 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `POST /session/{sessionID}/command` | 向会话发送命令由 AI 执行 | 已实现 |
| [Session 会话 / Message 消息](08-session.md) | `POST /session/{sessionID}/shell` | 在会话上下文中执行 shell 命令 | 已实现 |
| [Permission 权限](09-permission.md) | `GET /permission` | 列出待处理权限请求 | 已实现 |
| [Permission 权限](09-permission.md) | `POST /permission/{requestID}/reply` | 批准或拒绝权限请求 | 已实现 |
| [Question 问题](10-question.md) | `GET /question` | 列出待回答问题 | 已实现 |
| [Question 问题](10-question.md) | `POST /question/{requestID}/reply` | 回答问题 | 已实现 |
| [Question 问题](10-question.md) | `POST /question/{requestID}/reject` | 拒绝问题 | 已实现 |
| [Command 命令](11-command.md) | `GET /command` | 列出所有可用命令 | 已实现 |
| [File 文件](12-file.md) | `GET /file` | 列出指定路径下的文件与目录 | 已实现 |
| [File 文件](12-file.md) | `GET /file/content` | 读取文件内容 | 已实现 |
| [File 文件](12-file.md) | `GET /file/status` | 获取项目内文件 git 状态 | 已实现 |
| [Find 查找](13-find.md) | `GET /find` | 文本搜索（ripgrep） | 已实现 |
| [Find 查找](13-find.md) | `GET /find/file` | 按名称或模式搜索文件/目录 | 已实现 |
| [Find 查找](13-find.md) | `GET /find/symbol` | 符号搜索（LSP） | 已实现 |
| [LSP / Formatter / MCP](14-lsp-formatter-mcp.md) | `GET /lsp` | 获取 LSP 服务状态 | 已实现 |
| [LSP / Formatter / MCP](14-lsp-formatter-mcp.md) | `GET /formatter` | 获取 Formatter 状态 | 已实现 |
| [LSP / Formatter / MCP](14-lsp-formatter-mcp.md) | `GET /mcp` | 获取 MCP 服务状态 | 已实现 |
| [LSP / Formatter / MCP](14-lsp-formatter-mcp.md) | `POST /mcp` | 动态添加 MCP 服务 | 已实现 |
| [LSP / Formatter / MCP](14-lsp-formatter-mcp.md) | `POST /mcp/{name}/auth` | 启动 MCP OAuth | 已实现 |
| [LSP / Formatter / MCP](14-lsp-formatter-mcp.md) | `DELETE /mcp/{name}/auth` | 移除 MCP OAuth 凭据 | 已实现 |
| [LSP / Formatter / MCP](14-lsp-formatter-mcp.md) | `POST /mcp/{name}/auth/authenticate` | 启动 MCP OAuth 并等待回调 | 已实现 |
| [LSP / Formatter / MCP](14-lsp-formatter-mcp.md) | `POST /mcp/{name}/auth/callback` | 完成 MCP OAuth 回调 | 已实现 |
| [LSP / Formatter / MCP](14-lsp-formatter-mcp.md) | `POST /mcp/{name}/connect` | 连接 MCP 服务 | 已实现 |
| [LSP / Formatter / MCP](14-lsp-formatter-mcp.md) | `POST /mcp/{name}/disconnect` | 断开 MCP 服务 | 已实现 |
| [Agent & Skill](15-agent-skill.md) | `GET /agent` | 列出可用 AI 代理 | 已实现 |
| [Agent & Skill](15-agent-skill.md) | `GET /skill` | 列出可用技能 | 已实现 |
| [Logging 日志](16-logging.md) | `POST /log` | 向服务端写日志条目 | 已实现 |
| [Event 事件](17-event.md) | `GET /event` | 订阅服务端事件流（SSE） | 已实现 |
| [PTY 伪终端](18-pty.md) | `GET /pty` | 列出 PTY 会话 | 已实现 |
| [PTY 伪终端](18-pty.md) | `POST /pty` | 创建 PTY 会话 | 已实现 |
| [PTY 伪终端](18-pty.md) | `GET /pty/{ptyID}` | 获取 PTY 会话详情 | 已实现 |
| [PTY 伪终端](18-pty.md) | `PUT /pty/{ptyID}` | 更新 PTY 会话 | 已实现 |
| [PTY 伪终端](18-pty.md) | `DELETE /pty/{ptyID}` | 移除并终止 PTY 会话 | 已实现 |
| [PTY 伪终端](18-pty.md) | `GET /pty/{ptyID}/connect` | 建立 WebSocket 连接与 PTY 交互 | 已实现 |
| [TUI 界面控制](19-tui.md) | `POST /tui/append-prompt` | 向输入框追加内容 | 已实现 |
| [TUI 界面控制](19-tui.md) | `POST /tui/clear-prompt` | 清空输入框 | 已实现 |
| [TUI 界面控制](19-tui.md) | `POST /tui/submit-prompt` | 提交当前输入 | 已实现 |
| [TUI 界面控制](19-tui.md) | `POST /tui/open-help` | 打开帮助对话框 | 已实现 |
| [TUI 界面控制](19-tui.md) | `POST /tui/open-sessions` | 打开会话选择对话框 | 已实现 |
| [TUI 界面控制](19-tui.md) | `POST /tui/open-models` | 打开模型选择对话框 | 已实现 |
| [TUI 界面控制](19-tui.md) | `POST /tui/open-themes` | 打开主题选择对话框 | 已实现 |
| [TUI 界面控制](19-tui.md) | `POST /tui/execute-command` | 执行 TUI 命令 | 已实现 |
| [TUI 界面控制](19-tui.md) | `POST /tui/show-toast` | 显示 toast 通知 | 已实现 |
| [TUI 界面控制](19-tui.md) | `POST /tui/select-session` | 切换到指定会话 | 已实现 |
| [TUI 界面控制](19-tui.md) | `POST /tui/publish` | 发布 TUI 事件 | 已实现 |
| [TUI 界面控制](19-tui.md) | `GET /tui/control/next` | 获取下一个 TUI 控制请求 | 已实现 |
| [TUI 界面控制](19-tui.md) | `POST /tui/control/response` | 提交对 TUI 控制请求的响应 | 已实现 |
| [Experimental 实验性](20-experimental.md) | `GET /experimental/tool/ids` | 列出所有工具 ID | 已实现 |
| [Experimental 实验性](20-experimental.md) | `GET /experimental/tool` | 获取某 provider+model 的工具列表 | 已实现 |
| [Experimental 实验性](20-experimental.md) | `GET /experimental/resource` | 获取 MCP 资源 | 已实现 |
| [Experimental 实验性](20-experimental.md) | `GET /experimental/worktree` | 列出 worktree | 已实现 |
| [Experimental 实验性](20-experimental.md) | `POST /experimental/worktree` | 创建 worktree | 已实现 |
| [Experimental 实验性](20-experimental.md) | `DELETE /experimental/worktree` | 删除 worktree | 已实现 |
| [Experimental 实验性](20-experimental.md) | `POST /experimental/worktree/reset` | 重置 worktree 分支 | 已实现 |

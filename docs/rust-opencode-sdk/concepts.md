# OpenCode 核心概念

> 理解 OpenCode Server 的核心概念，有助于更好地使用 SDK

## 架构概览

OpenCode 采用 **客户端-服务器架构**：

```
┌─────────────────────────────────────────────────────────────┐
│                        客户端层                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │   TUI    │  │   IDE    │  │  Desktop │  │   SDK    │   │
│  │  终端界面 │  │  插件    │  │   应用   │  │ (本库)   │   │
│  └─────┬────┘  └─────┬────┘  └─────┬────┘  └─────┬────┘   │
└────────┼────────────┼────────────┼────────────┼───────────┘
         │            │            │            │
         └────────────┴────────────┴────────────┘
                              │
                    ┌─────────▼─────────┐
                    │  OpenCode Server  │
                    │      (Go)         │
                    │   端口: 4096      │
                    └───────────────────┘
```

- **Server**: 基于 Go 的 HTTP 服务器，提供 REST API 和 SSE 事件
- **客户端**: TUI、IDE 插件、桌面应用、或自定义 SDK 客户端
- **通信**: 通过 HTTP/WebSocket 进行通信

---

## 核心概念

### 1. 会话 (Session)

会话是与 AI 的对话上下文，包含消息历史和状态。

```rust
let session = client.session()
    .create(CreateSessionOptions {
        title: Some("分析项目结构".into()),
        ..Default::default()
    })
    .await?;
```

**关键特性**:
- 每个 Session 独立维护上下文
- 支持嵌套子会话（用于任务分支）
- 可持久化存储在 SQLite 数据库
- 支持分享（生成公开 URL）

### 2. 代理 (Agent)

代理是具有特定工具访问权限和系统提示的专门化 AI 助手。

#### 代理类型

```
┌─────────────────────────────────────────────────────────┐
│                    Primary Agents                        │
│  ┌─────────────┐  ┌─────────────┐                       │
│  │   Build     │  │    Plan     │                       │
│  │  全工具权限  │  │  只读分析   │                       │
│  │  (默认)     │  │  (需批准)   │                       │
│  └─────────────┘  └─────────────┘                       │
│        │                    │                            │
│        └──────────┬─────────┘                            │
│                   ▼                                      │
│            用户直接交互 (Tab 切换)                        │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│                   Subagents                              │
│  ┌─────────────┐  ┌─────────────┐                       │
│  │  General    │  │  Explore    │                       │
│  │  多步骤任务  │  │  快速探索   │                       │
│  │  (@mention) │  │  (@mention) │                       │
│  └─────────────┘  └─────────────┘                       │
│                                                           │
│  由 Primary Agent 调用或手动 @ 提及                       │
└─────────────────────────────────────────────────────────┘
```

#### 内置主代理 (Primary Agents)

| 代理 | 模式 | 工具权限 | 使用场景 |
|------|------|----------|----------|
| **Build** | `primary` | 全部 | 开发工作，需要完整文件和系统操作 |
| **Plan** | `primary` | 只读 (ask) | 代码分析、架构规划、建议修改 |

#### 内置子代理 (Subagents)

| 代理 | 模式 | 工具权限 | 使用场景 |
|------|------|----------|----------|
| **General** | `subagent` | 除 todo 外全部 | 复杂多步骤任务，并行执行 |
| **Explore** | `subagent` | 只读 | 快速搜索文件、代码探索 |

#### 代码示例

**列出所有可用代理**:

```rust
use opencode::Opencode;

let client = Opencode::new("http://127.0.0.1:4096");

let agents = client.agent().list().await?;

for agent in agents {
    println!("{} - {}", agent.id, agent.name);
    if let Some(desc) = agent.description {
        println!("  {}", desc);
    }
}
```

**使用 Plan 代理进行代码分析**:

```rust
use opencode::{Opencode, SendMessageOptions, Part, CreateSessionOptions};

let client = Opencode::new("http://127.0.0.1:4096");

// 创建会话
let session = client.session()
    .create(CreateSessionOptions {
        title: Some("代码审查".into()),
        ..Default::default()
    })
    .await?;

// 使用 plan 代理（只读，不会修改代码）
let response = client.session().send_message(
    &session.id,
    SendMessageOptions {
        agent: Some("plan".into()),
        parts: vec![Part {
            part_type: "text".into(),
            text: Some("分析 src/ 目录下的代码结构，找出潜在问题".into()),
            ..Default::default()
        }],
        ..Default::default()
    },
).await?;

// Plan 代理只会分析，不会执行 edit/bash 等修改操作
```

**使用 Build 代理进行开发**:

```rust
// 使用 build 代理（默认，拥有所有工具权限）
let response = client.session().send_message(
    &session.id,
    SendMessageOptions {
        agent: Some("build".into()),
        parts: vec![Part {
            part_type: "text".into(),
            text: Some("添加一个用户认证模块，包含登录和注册功能".into()),
            ..Default::default()
        }],
        ..Default::default()
    },
).await?;

// Build 代理可以执行 edit、write、bash 等操作
```

**在消息中提及子代理**:

```rust
// 让 General 子代理执行特定任务
let response = client.session().send_message(
    &session.id,
    SendMessageOptions {
        parts: vec![Part {
            part_type: "text".into(),
            text: Some("@general 帮我重构一下 auth.rs 文件".into()),
            ..Default::default()
        }],
        ..Default::default()
    },
).await?;
```

**使用不同模型配置代理**:

```rust
use opencode::{Model, SendMessageOptions, Part};

// Plan 使用快速模型进行分析
let analysis = client.session().send_message(
    &session_id,
    SendMessageOptions {
        agent: Some("plan".into()),
        model: Some(Model {
            provider_id: "anthropic".into(),
            model_id: "claude-haiku-4".into(),  // 快速模型
        }),
        parts: vec![Part {
            part_type: "text".into(),
            text: Some("快速检查代码风格".into()),
            ..Default::default()
        }],
        ..Default::default()
    },
).await?;

// Build 使用强大模型进行实现
let implementation = client.session().send_message(
    &session_id,
    SendMessageOptions {
        agent: Some("build".into()),
        model: Some(Model {
            provider_id: "anthropic".into(),
            model_id: "claude-sonnet-4".into(),  // 强大模型
        }),
        parts: vec![Part {
            part_type: "text".into(),
            text: Some("根据之前的分析实现功能".into()),
            ..Default::default()
        }],
        ..Default::default()
    },
).await?;
```

#### 代理配置示例

在 `opencode.json` 中配置代理：

```json
{
  "$schema": "https://opencode.ai/config.json",
  "agent": {
    "build": {
      "mode": "primary",
      "model": "anthropic/claude-sonnet-4",
      "temperature": 0.3,
      "tools": {
        "bash": true,
        "edit": true,
        "write": true
      }
    },
    "plan": {
      "mode": "primary",
      "model": "anthropic/claude-haiku-4",
      "temperature": 0.1,
      "tools": {
        "bash": false,
        "edit": false,
        "write": false
      },
      "permission": {
        "edit": "ask",
        "bash": "ask"
      }
    },
    "reviewer": {
      "mode": "subagent",
      "description": "代码审查专家，专注于安全和性能",
      "model": "anthropic/claude-sonnet-4",
      "temperature": 0.1,
      "prompt": "{file:./prompts/code-review.txt}",
      "tools": {
        "write": false,
        "edit": false,
        "bash": false
      }
    }
  }
}
```

### 3. 工具 (Tools)

工具是 LLM 可以调用的功能，用于操作代码库。

#### 内置工具

| 工具 | 说明 | 权限控制 |
|------|------|----------|
| `bash` | 执行 Shell 命令 | `permission.bash` |
| `edit` | 精确字符串替换编辑文件 | `permission.edit` |
| `write` | 创建新文件或覆盖文件 | `permission.edit` |
| `read` | 读取文件内容 | 无需权限 |
| `grep` | 正则表达式搜索文件内容 | 无需权限 |
| `glob` | 使用模式匹配查找文件 | 无需权限 |
| `lsp` | LSP 代码智能（定义、引用、悬停等） | `permission.lsp` |
| `webfetch` | 获取网页内容 | `permission.webfetch` |
| `todowrite` | 管理任务列表 | 无需权限 |
| `todoread` | 读取任务列表 | 无需权限 |

```json
{
  "permission": {
    "bash": "ask",
    "edit": "allow",
    "webfetch": "deny"
  }
}
```

### 4. 提供商 (Provider)

提供商是 LLM 服务的抽象层，支持 75+ 提供商。

| 提供商 | 说明 |
|--------|------|
| `anthropic` | Claude 系列 |
| `openai` | GPT 系列 |
| `google` | Gemini 系列 |
| `opencode` | Zen 托管模型 |
| `ollama` | 本地模型 |
| `...` | 更多 |

```rust
// 指定模型
let response = client.session().send_message(
    &session_id,
    SendMessageOptions {
        model: Some(Model {
            provider_id: "anthropic".into(),
            model_id: "claude-sonnet-4".into(),
        }),
        parts: vec![...],
        ..Default::default()
    },
).await?;
```

### 5. 权限 (Permission)

控制系统对工具调用的处理方式。

| 权限值 | 说明 |
|--------|------|
| `allow` | 自动允许，无需确认 |
| `ask` | 每次执行前询问用户 |
| `deny` | 禁用工具 |

### 6. 项目 (Project)

项目是 OpenCode 管理的工作目录。

```rust
// 获取当前项目
let project = client.project().current().await?;

// 列出所有项目
let projects = client.project().list().await?;
```

### 7. 消息 (Message)

消息是会话中的对话单元。

```rust
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,  // "user" | "assistant"
    pub created_at: String,
}
```

每条消息由多个 **Part** 组成：
- `text`: 文本内容
- `tool`: 工具调用
- `image`: 图片内容

### 8. 事件 (Events)

服务器通过 SSE 推送实时事件。

```rust
let mut stream = client.event().subscribe().await?;

while let Some(event) = stream.next().await {
    match event.event_type.as_str() {
        "server.connected" => println!("已连接"),
        "session.created" => println!("会话已创建"),
        "message.created" => println!("新消息"),
        _ => {}
    }
}
```

**常见事件类型**:
- `server.connected` - 服务器连接
- `session.created` - 会话创建
- `session.deleted` - 会话删除
- `message.created` - 消息创建
- `message.updated` - 消息更新

---

## 数据流

```
用户输入
    │
    ▼
┌─────────────┐
│  Session    │
│   上下文    │
└─────┬───────┘
      │
      ▼
┌─────────────┐     ┌──────────────┐
│   Agent     │────>│   Provider   │
│ (选择工具)  │     │  (LLM 调用)  │
└─────┬───────┘     └──────────────┘
      │                     │
      ▼                     ▼
┌─────────────┐     ┌──────────────┐
│   Tools     │     │    Response  │
│ (执行操作)  │<────│  (AI 回复)   │
└─────────────┘     └──────────────┘
      │
      ▼
  更新上下文
```

---

## 配置文件

OpenCode 使用 `opencode.json` 进行配置：

```json
{
  "$schema": "https://opencode.ai/config.json",
  "model": "anthropic/claude-sonnet-4",
  "agent": {
    "build": {
      "tools": {
        "bash": true,
        "edit": true
      }
    }
  },
  "permission": {
    "bash": "ask",
    "edit": "allow"
  }
}
```

---

## 相关链接

- [Agents 文档](https://opencode.ai/docs/agents/)
- [Tools 文档](https://opencode.ai/docs/tools/)
- [Server API](https://opencodecn.com/docs/server)
- [配置参考](https://opencode.ai/docs/config/)

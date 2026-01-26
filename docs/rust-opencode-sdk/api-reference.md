# API 参考

## Client

### 构造函数

| 方法 | 说明 |
|------|------|
| `Opencode::new(base_url: impl Into<String>)` | 创建新客户端 |
| `Opencode::with_config(config: Config)` | 使用配置创建客户端 |

### API 访问器

| 访问器 | 返回类型 | 说明 |
|--------|----------|------|
| `client.global()` | `GlobalApi<'_>` | 全局 API |
| `client.session()` | `SessionApi<'_>` | 会话 API |
| `client.find()` | `FindApi<'_>` | 搜索 API |
| `client.file()` | `FileApi<'_>` | 文件 API |
| `client.project()` | `ProjectApi<'_>` | 项目 API |
| `client.path()` | `PathApi<'_>` | 路径 API |
| `client.vcs()` | `VcsApi<'_>` | VCS API |
| `client.config_api()` | `ConfigApi<'_>` | 配置 API |
| `client.agent()` | `AgentApi<'_>` | 代理 API |
| `client.provider()` | `ProviderApi<'_>` | 提供商 API |
| `client.command()` | `CommandApi<'_>` | 命令 API |
| `client.auth()` | `AuthApi<'_>` | 认证 API |
| `client.tui()` | `TuiApi<'_>` | TUI API |
| `client.event()` | `EventApi<'_>` | 事件 API (SSE) |

---

## Global API

`client.global()`

### 方法

| 方法 | HTTP 端点 | 返回类型 | 说明 |
|------|-----------|----------|------|
| `health().await` | `GET /global/health` | `Health` | 健康检查 |

### 类型

```rust
pub struct Health {
    pub healthy: bool,
    pub version: String,
}
```

---

## Session API

`client.session()`

### 方法

| 方法 | HTTP 端点 | 返回类型 | 说明 |
|------|-----------|----------|------|
| `list().await` | `GET /session` | `Vec<Session>` | 列出所有会话 |
| `get(id).await` | `GET /session/:id` | `Session` | 获取会话详情 |
| `create(opts).await` | `POST /session` | `Session` | 创建新会话 |
| `update(id, opts).await` | `PATCH /session/:id` | `Session` | 更新会话 |
| `delete(id).await` | `DELETE /session/:id` | `bool` | 删除会话 |
| `children(id).await` | `GET /session/:id/children` | `Vec<Session>` | 获取子会话 |
| `abort(id).await` | `POST /session/:id/abort` | `bool` | 中止会话 |
| `share(id).await` | `POST /session/:id/share` | `Session` | 分享会话 |
| `unshare(id).await` | `DELETE /session/:id/share` | `Session` | 取消分享 |
| `messages(id).await` | `GET /session/:id/message` | `Vec<MessageResponse>` | 获取消息列表 |
| `send_message(id, opts).await` | `POST /session/:id/message` | `MessageResponse` | 发送消息 |
| `command(id, opts).await` | `POST /session/:id/command` | `MessageResponse` | 执行命令 |
| `shell(id, opts).await` | `POST /session/:id/shell` | `MessageResponse` | 运行 shell |
| `summarize(id, opts).await` | `POST /session/:id/summarize` | `bool` | 总结会话 |
| `revert(id, opts).await` | `POST /session/:id/revert` | `bool` | 撤销消息 |
| `unrevert(id).await` | `POST /session/:id/unrevert` | `bool` | 恢复撤销 |

### 类型

```rust
pub struct Session {
    pub id: String,
    pub title: Option<String>,
    pub parent_id: Option<String>,
    pub created_at: String,
}

pub struct CreateSessionOptions {
    pub parent_id: Option<String>,
    pub title: Option<String>,
}

pub struct UpdateSessionOptions {
    pub title: Option<String>,
}

pub struct SendMessageOptions {
    pub message_id: Option<String>,
    pub model: Option<Model>,
    pub agent: Option<String>,
    pub no_reply: Option<bool>,
    pub system: Option<String>,
    pub parts: Vec<Part>,
}

pub struct Model {
    pub provider_id: String,
    pub model_id: String,
}

pub struct MessageResponse {
    pub info: Message,
    pub parts: Vec<Part>,
}

pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub created_at: String,
}
```

---

## Find API

`client.find()`

### 方法

| 方法 | HTTP 端点 | 返回类型 | 说明 |
|------|-----------|----------|------|
| `text(pattern).await` | `GET /find?pattern=` | `Vec<TextMatch>` | 搜索文本 |
| `files(query, type, dir, limit).await` | `GET /find/file?query=` | `Vec<String>` | 查找文件 |
| `symbols(query).await` | `GET /find/symbol?query=` | `Vec<Symbol>` | 查找符号 |

### 类型

```rust
pub struct TextMatch {
    pub path: String,
    pub lines: String,
    pub line_number: usize,
    pub absolute_offset: usize,
    pub submatches: Vec<Submatch>,
}

pub struct Symbol {
    pub name: String,
    pub path: Option<String>,
    pub kind: Option<String>,
}
```

---

## File API

`client.file()`

### 方法

| 方法 | HTTP 端点 | 返回类型 | 说明 |
|------|-----------|----------|------|
| `read(path).await` | `GET /file/content?path=` | `FileContent` | 读取文件 |
| `status().await` | `GET /file/status` | `Vec<File>` | 文件状态 |

### 类型

```rust
pub struct FileContent {
    pub r#type: String,  // "raw" | "patch"
    pub content: String,
}

pub struct File {
    pub path: String,
    pub r#type: String,
}
```

---

## Project API

`client.project()`

### 方法

| 方法 | HTTP 端点 | 返回类型 | 说明 |
|------|-----------|----------|------|
| `list().await` | `GET /project` | `Vec<Project>` | 列出所有项目 |
| `current().await` | `GET /project/current` | `Project` | 当前项目 |

---

## Path API

`client.path()`

### 方法

| 方法 | HTTP 端点 | 返回类型 | 说明 |
|------|-----------|----------|------|
| `get().await` | `GET /path` | `Path` | 获取当前路径 |

---

## VCS API

`client.vcs()`

### 方法

| 方法 | HTTP 端点 | 返回类型 | 说明 |
|------|-----------|----------|------|
| `get().await` | `GET /vcs` | `VcsInfo` | 获取 VCS 信息 |

---

## Config API

`client.config_api()`

### 方法

| 方法 | HTTP 端点 | 返回类型 | 说明 |
|------|-----------|----------|------|
| `get().await` | `GET /config` | `Config` | 获取配置 |
| `providers().await` | `GET /config/providers` | `ProvidersInfo` | 提供商列表 |

---

## Agent API

`client.agent()`

### 方法

| 方法 | HTTP 端点 | 返回类型 | 说明 |
|------|-----------|----------|------|
| `list().await` | `GET /agent` | `Vec<Agent>` | 列出所有代理 |

---

## Auth API

`client.auth()`

### 方法

| 方法 | HTTP 端点 | 返回类型 | 说明 |
|------|-----------|----------|------|
| `set(id, opts).await` | `PUT /auth/:id` | `bool` | 设置认证 |

---

## Event API

`client.event()`

### 方法

| 方法 | HTTP 端点 | 返回类型 | 说明 |
|------|-----------|----------|------|
| `subscribe().await` | `GET /event` | `Stream<Event>` | SSE 事件流 |

### 类型

```rust
pub struct Event {
    pub event_type: String,
    pub properties: serde_json::Value,
}
```

### 使用示例

```rust
let mut stream = client.event().subscribe().await?;
while let Some(event) = stream.next().await {
    match event {
        Ok(e) => println!("Event: {}", e.event_type),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

---

## TUI API

`client.tui()`

### 方法

| 方法 | HTTP 端点 | 返回类型 | 说明 |
|------|-----------|----------|------|
| `append_prompt(text).await` | `POST /tui/append-prompt` | `bool` | 追加文本到提示 |
| `open_help().await` | `POST /tui/open-help` | `bool` | 打开帮助 |
| `open_sessions().await` | `POST /tui/open-sessions` | `bool` | 打开会话选择器 |
| `submit_prompt().await` | `POST /tui/submit-prompt` | `bool` | 提交提示 |
| `clear_prompt().await` | `POST /tui/clear-prompt` | `bool` | 清除提示 |
| `show-toast(msg, variant).await` | `POST /tui/show-toast` | `bool` | 显示通知 |

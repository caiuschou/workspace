# OpenCode.open 方法设计

> 设计一个 `OpenCode::open` 方法，用于自动启动 OpenCode serve 并返回已连接的客户端。
>
> 对标 TypeScript SDK 的 `createOpencode()`。

## 1. 目标

提供类似 `createOpencode()` 的一站式入口：

- **若 serve 已在运行**：直接返回连接该实例的 `Client`
- **若 serve 未运行**：启动 `opencode serve` 进程，等待就绪后返回 `Client`
- **返回值**：`(Client, Option<ServerHandle>)`，当由本 SDK 启动的 serve 时，`ServerHandle` 用于关闭

## 2. 参考：TypeScript 实现

```
createOpencode() 流程：
1. 若 autoStart=false → 仅返回 client，server=null
2. 检测 opencode 命令是否在 PATH 中（which/where）
3. 健康检查 baseUrl/global/health
4. 若健康 → 返回 (client, null)
5. 若非健康 → spawn("opencode", ["serve", "--port", port, "--hostname", hostname])
6. 轮询健康检查直到就绪（默认 30s 超时）
7. 返回 (client, ServerHandle)，ServerHandle 含 pid 与 shutdown()
```

## 3. API 设计

### 3.1 主入口

```rust
/// Opens OpenCode: connects to existing server or starts one.
///
/// If the server is already running at the given address, returns a client
/// connected to it. Otherwise spawns `opencode serve` and waits for it to be ready.
///
/// # Example
///
/// ```rust,no_run
/// use opencode_sdk::OpenCode;
///
/// #[tokio::main]
/// async fn main() -> Result<(), opencode_sdk::Error> {
///     let (client, server) = OpenCode::open(OpenOptions::default()).await?;
///     let health = client.health().await?;
///     println!("version: {}", health.version);
///     if let Some(s) = server {
///         s.shutdown().await?;  // shutdown if we started it
///     }
///     Ok(())
/// }
/// ```
pub async fn open(options: OpenOptions) -> Result<(Client, Option<ServerHandle>), Error>
```

### 3.2 配置选项

```rust
/// Options for OpenCode::open.
#[derive(Debug, Clone)]
pub struct OpenOptions {
    /// Server hostname (default: "127.0.0.1").
    pub hostname: String,

    /// Server port (default: 4096).
    pub port: u16,

    /// Whether to auto-start the server if not running (default: true).
    pub auto_start: bool,

    /// Command to run for server (default: "opencode").
    /// Can be full path or command name in PATH.
    pub command: String,

    /// Extra arguments for `opencode serve` (e.g. ["--cors", "http://localhost:3000"]).
    pub server_args: Vec<String>,

    /// Timeout for server health check when probing (ms).
    pub health_check_timeout_ms: u64,

    /// Max time to wait for server to become ready after spawn (ms).
    pub startup_timeout_ms: u64,

    /// Working directory for the server process (default: current dir).
    pub working_directory: Option<PathBuf>,
}

impl Default for OpenOptions {
    fn default() -> Self { ... }
}
```

### 3.3 ServerHandle

```rust
/// Handle to a server process started by this SDK.
///
/// Use [ServerHandle::shutdown] to gracefully terminate the server.
#[derive(Debug)]
pub struct ServerHandle {
    pid: u32,
}

impl ServerHandle {
    /// Returns the process ID.
    pub fn pid(&self) -> u32 { ... }

    /// Gracefully shuts down the server (SIGTERM on Unix, taskkill on Windows).
    /// Best-effort: errors (e.g. process already exited) are ignored.
    pub fn shutdown(&self) { ... }
}
```

## 4. 执行流程

```
OpenCode::open(options)
│
├─ auto_start == false
│  └─ return (Client::new(base_url), None)
│
├─ 1. 检测命令
│  └─ which opencode / where opencode
│     └─ 失败 → Error::CommandNotFound(command)
│
├─ 2. 健康检查
│  └─ GET {base_url}/global/health
│     └─ 成功 → return (Client::new(base_url), None)
│
├─ 3. 启动进程
│  └─ Command::new(cmd)
│        .args(["serve", "--port", port, "--hostname", hostname, ...args])
│        .stdin(Stdio::null())
│        .stdout(Stdio::null())
│        .stderr(Stdio::null())
│        .spawn()
│     └─ 失败 → Error::SpawnFailed
│
├─ 4. 轮询就绪
│  └─ loop { health_check(); sleep(interval); }
│     └─ 超时 → kill process, Error::StartupTimeout
│
└─ 5. return (Client::new(base_url), Some(ServerHandle { pid }))
```

## 5. 错误类型扩展

```rust
#[derive(Error, Debug)]
pub enum Error {
    // ... 现有变体 ...

    /// OpenCode command not found in PATH.
    #[error("opencode command not found: {0}. Install from https://opencode.ai/install")]
    CommandNotFound(String),

    /// Failed to spawn server process.
    #[error("failed to spawn opencode serve: {0}")]
    SpawnFailed(#[source] std::io::Error),

    /// Server did not become ready within timeout.
    #[error("opencode server at {url} did not become ready within {timeout_ms}ms")]
    StartupTimeout { url: String, timeout_ms: u64 },
}
```

## 6. 跨平台

| 平台   | 命令检测        | 进程终止                 |
|--------|-----------------|--------------------------|
| Unix   | `which opencode`| `kill(pid, SIGTERM)`     |
| Windows| `where opencode`| `taskkill /PID pid /T`   |

使用 `std::process::Command` 和 `#[cfg(unix)]` / `#[cfg(windows)]` 分别实现。

## 7. 依赖

- `tokio::process::Command`：异步启动进程（或 `std::process::Command` + `spawn_blocking`）
- 无新增 crate：用现有 `reqwest` 做健康检查

可选：若希望更精确的进程控制，可引入 `nix`（Unix signal）或 `sysinfo`，初期可先用 `std::process::Child::kill()` 简化。

## 8. 模块划分建议

```
opencode-sdk/src/
├── lib.rs
├── client.rs
├── error.rs
├── open.rs           # OpenCode::open, OpenOptions, ServerHandle
├── server/
│   ├── mod.rs
│   ├── detect.rs     # 命令检测 (which/where)
│   ├── spawn.rs      # 启动 opencode serve
│   └── shutdown.rs   # 进程终止
```

## 9. 使用示例

```rust
// 自动启动（默认）
let (client, server) = OpenCode::open(OpenOptions::default()).await?;

// 仅连接已有 serve
let (client, _) = OpenCode::open(
    OpenOptions::default().auto_start(false)
).await?;

// 自定义端口与命令
let (client, server) = OpenCode::open(
    OpenOptions::default()
        .port(5000)
        .command("/usr/local/bin/opencode")
        .startup_timeout_ms(60_000)
).await?;

// 用完后关闭
if let Some(s) = server {
    s.shutdown().await?;
}
```

## 10. 任务拆分

| 任务                         | 状态 |
|------------------------------|------|
| 扩展 Error 枚举              | 完成 |
| 实现 detect 模块（which/where） | 完成 |
| 实现 spawn 逻辑              | 完成 |
| 实现 shutdown 逻辑           | 完成 |
| 实现 OpenOptions + Default   | 完成 |
| 实现 OpenCode::open 主流程   | 完成 |
| 单元/集成测试                | 待办 |

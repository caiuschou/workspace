# opencode-sdk

Rust SDK for [OpenCode Server](https://opencodecn.com/docs/server) API.

独立 workspace，不依赖主仓库的 workspace 配置。

## 快速开始

```toml
[dependencies]
opencode-sdk = "0.1"
```

```rust
use opencode_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("http://127.0.0.1:4096");
    let health = client.health().await?;
    println!("Server version: {}", health.version);
    Ok(())
}
```

## 开发

```bash
cd opencode-sdk
cargo build
cargo test
```

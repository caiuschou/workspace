# 开发指南

## 构建

```bash
# 构建 workspace
cargo build

# 发布构建
cargo build --release
```

## 运行

```bash
# 运行 MCP Server
cargo run --bin mcp-server

# 运行 CLI
cargo run --bin gitlab-mcp -- project list

# 带环境变量运行
GITLAB_URL="https://gitlab.com" GITLAB_TOKEN="glpat-xxx" cargo run --bin mcp-server
```

## 测试

```bash
# 运行所有测试
cargo test

# 运行单个 crate 测试
cargo test -p mcp-server
cargo test -p mcp-client

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_name
```

## 项目结构

```
gitlab-mcp/
├── crates/
│   ├── mcp-server/    # MCP Server
│   └── mcp-client/    # CLI Client
└── docs/
    └── solution.md    # 解决方案文档
```

## 添加新工具

1. 在 `crates/mcp-server/src/tools/` 创建新模块
2. 在 `crates/mcp-server/src/tools/mod.rs` 注册工具
3. 在 `crates/mcp-client/src/commands/` 创建对应 CLI 命令
4. 在 `crates/mcp-client/src/commands/mod.rs` 注册命令

## 代码风格

```bash
# 格式化代码
cargo fmt

# 检查代码风格
cargo fmt --check
```

## Lint

```bash
# 运行 clippy
cargo clippy

# 修复可自动修复的问题
cargo clippy --fix
```

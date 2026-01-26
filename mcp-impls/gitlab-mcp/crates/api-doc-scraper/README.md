# GitLab API Documentation Scraper

从 `https://docs.gitlab.com/ee/api/` 抓取 GitLab REST API 文档并保存为 Markdown 格式。

## 功能特性

- 抓取约 140+ 个 API 资源页面
- 转换为 Markdown 格式
- 按类别组织（Project、Group、Standalone、Templates）
- 速率限制（1 请求/秒）
- 自动重试机制
- 进度条显示

## 文件结构

```
crates/api-doc-scraper/
├── Cargo.toml          # Crate 配置
├── README.md           # 本文件
└── src/
    ├── main.rs         # CLI 入口和主抓取逻辑
    ├── lib.rs          # 库入口
    ├── client.rs       # HTTP 客户端（带限流）
    ├── parser.rs       # HTML 解析和 Markdown 转换
    ├── resources.rs    # API 资源列表定义
    └── error.rs        # 错误类型
```

## 编译

```bash
# 需要先确保 Rust 环境配置正确
cargo build --release --bin api-doc-scraper
```

## 使用方法

```bash
# 抓取所有资源
cargo run --bin api-doc-scraper

# 抓取单个资源
cargo run --bin api-doc-scraper -- --resource issues

# 抓取特定类别
cargo run --bin api-doc-scraper -- --category project

# 干运行（显示将要抓取的内容）
cargo run --bin api-doc-scraper -- --dry-run

# 指定输出目录
cargo run --bin api-doc-scraper -- --output-path /path/to/output
```

## 输出结构

```
docs/gitlab-api/
├── README.md              # 总索引
├── project/               # Project 资源
│   ├── issues.md
│   ├── merge_requests.md
│   ├── pipelines.md
│   └── ...
├── group/                 # Group 资源
│   ├── epics.md
│   ├── groups.md
│   └── ...
├── standalone/            # Standalone 资源
│   ├── users.md
│   ├── projects.md
│   └── ...
└── templates/             # Templates 资源
    ├── dockerfiles.md
    ├── gitignores.md
    └── licenses.md
```

## 依赖

- `tokio` - 异步运行时
- `reqwest` - HTTP 客户端
- `scraper` - HTML 解析
- `html2md` - HTML 到 Markdown 转换
- `governor` - 速率限制
- `indicatif` - 进度条
- `clap` - CLI 参数解析
- `tracing` - 日志

## 注意事项

1. **编译环境**: Windows 上需要正确配置 MSVC 或 GNU 工具链
2. **速率限制**: 默认 1 请求/秒，避免对 GitLab 服务器造成压力
3. **网络问题**: 内置重试机制，最多重试 3 次

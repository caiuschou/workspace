# GitHub Webhook 集成方案

> **前置阅读**: [Workspace API 架构设计](../workspace-api-architecture.md) - 通用架构、配置管理、错误处理、可观测性等

## 概述

本文档描述 `workspace-api` 如何接收和处理 GitHub Webhook 通知。

## 架构设计

```
                                    ┌─────────────────┐
                                    │   Prometheus    │
                                    └────────▲────────┘
                                             │ metrics
┌────────┐     ┌──────────────────────────────────────────────────┐
│ GitHub │────>│                  workspace-api                    │
└────────┘     │  ┌──────────┐   ┌──────────┐   ┌──────────────┐  │
               │  │   Rate   │──>│  Verify  │──>│   Idempotent │  │
               │  │  Limiter │   │Signature │   │    Check     │  │
               │  └──────────┘   └──────────┘   └──────┬───────┘  │
               │                                       │          │
               │                    ┌──────────────────▼────────┐ │
               │  202 Accepted <────│      Event Queue          │ │
               │                    │  (tokio channel / Redis)  │ │
               │                    └──────────────┬────────────┘ │
               └───────────────────────────────────│──────────────┘
                                                   │
                              ┌─────────────────────────────────────┐
                              │         Async Workers               │
                              │  ┌─────────┐ ┌─────────┐ ┌────────┐│
                              │  │  Push   │ │   PR    │ │ Issues ││
                              │  │ Handler │ │ Handler │ │Handler ││
                              │  └─────────┘ └─────────┘ └────────┘│
                              └─────────────────────────────────────┘
```

**核心设计原则**:
- **快速响应**: 立即返回 202 Accepted，避免 GitHub 10 秒超时
- **异步处理**: 事件入队后异步处理，提高吞吐量
- **幂等性**: 基于 `X-GitHub-Delivery` 去重，防止重复处理
- **可扩展**: 事件分发器支持注册新的处理器

## 依赖配置

在通用依赖基础上，添加 Webhook 特定依赖：

```toml
# crates/workspace-api/Cargo.toml

[dependencies]
# ... 通用依赖见 workspace-api-architecture.md

# Webhook 签名验证
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"

# 异步 trait
async-trait = "0.1"
```

## 文件结构

```
crates/workspace-api/src/
├── ...                          # 通用模块 (见架构文档)
└── webhooks/
    ├── mod.rs                   # Webhook 路由入口
    └── github/
        ├── mod.rs
        ├── models.rs            # 数据模型
        ├── verify.rs            # 签名验证
        ├── handler.rs           # 请求处理
        ├── dispatcher.rs        # 事件分发
        ├── idempotency.rs       # 幂等性检查
        └── handlers/            # 具体事件处理器
            ├── mod.rs
            ├── ping.rs
            ├── push.rs
            ├── pull_request.rs
            └── issues.rs
```

## 路由设计

| 路由 | 方法 | 描述 |
|------|------|------|
| `/webhooks/github` | POST | 接收 GitHub Webhook |
| `/webhooks/github/health` | GET | Webhook 端点健康检查 |

## 文档目录

| 文档 | 描述 |
|------|------|
| [数据模型](models.md) | Webhook 请求头、通用模型、事件模型 |
| [安全验证](security.md) | 签名验证、错误类型定义 |
| [事件分发](dispatcher.md) | 事件分发器、幂等性检查 |
| [请求处理](handler.md) | Webhook 主处理器 |
| [事件处理器](event-handlers.md) | 具体事件处理器示例 |
| [指标监控](metrics.md) | Webhook 指标、告警规则 |
| [路由集成](integration.md) | 路由集成、配置管理 |
| [测试指南](testing.md) | 安全检查、测试、后续扩展 |

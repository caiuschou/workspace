# MCP Rust Website

MCP Rust 项目的官方文档和状态展示网站。

## 技术栈

- **Next.js** 14.2.3
- **React** 18
- **TypeScript** 5
- **Tailwind CSS** 3.4.1

## 目录结构

```
page/
├── app/                        # Next.js App Router
│   ├── layout.tsx              # 全局布局（导航栏、Footer）
│   ├── page.tsx                # 首页
│   ├── globals.css             # 全局样式
│   ├── components/             # 共享组件
│   │   ├── StatusBadge.tsx     # 状态徽章
│   │   ├── CodeBlock.tsx       # 代码块
│   │   ├── FeatureCard.tsx     # 功能卡片
│   │   ├── DocCard.tsx         # 文档索引卡片
│   │   ├── Table.tsx           # 通用表格
│   │   └── index.ts            # 组件导出
│   ├── docs/                   # 文档页面
│   │   ├── layout.tsx          # 文档布局（侧边导航）
│   │   ├── page.tsx            # 文档索引
│   │   ├── quickstart/         # 快速开始
│   │   ├── architecture/       # 架构概览
│   │   ├── http-sse/           # HTTP/SSE 传输
│   │   ├── websocket/          # WebSocket 传输
│   │   ├── legacy-sse/         # 旧版 SSE
│   │   └── auth/               # OAuth 认证
│   └── dev/
│       └── page.tsx            # 开发进度
├── package.json
├── next.config.mjs             # Next.js 配置（standalone 输出）
├── tailwind.config.ts
├── tsconfig.json
└── postcss.config.js
```

## 开发命令

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 构建生产版本
npm run build

# 启动生产服务器
npm run start

# 代码检查
npm run lint
```

## 页面路由

| 路由 | 说明 |
|------|------|
| `/` | 首页，项目介绍和状态摘要 |
| `/docs` | 文档索引 |
| `/docs/quickstart` | 快速开始指南 |
| `/docs/architecture` | 架构概览 |
| `/docs/http-sse` | HTTP/SSE 传输文档 |
| `/docs/websocket` | WebSocket 传输文档 |
| `/docs/legacy-sse` | 旧版 SSE 兼容文档 |
| `/docs/auth` | OAuth 认证文档 |
| `/dev` | 开发进度和功能对比 |

## 部署

### 自托管 (Docker)

配置已启用 `standalone` 输出模式，支持 Docker 部署：

```bash
# 构建
npm run build

# standalone 输出在 .next/standalone/
```

### Vercel

直接部署到 Vercel，无需额外配置。

## 组件说明

| 组件 | 用途 |
|------|------|
| `StatusBadge` | 显示状态徽章（完成/进行中/未开始） |
| `CodeBlock` | 代码块展示，支持语法高亮 |
| `FeatureCard` | 功能特性卡片 |
| `DocCard` | 文档索引卡片，带链接 |
| `Table` | 通用表格组件 |

# SDK 参考

OpenCode SDK 是一个类型安全的 JavaScript/TypeScript 客户端，用于程序化控制 OpenCode 服务器。

## 安装

```bash
npm install @opencode-ai/sdk
```

## 快速开始

### 创建完整实例（服务器 + 客户端）

```typescript
import { createOpencode } from "@opencode-ai/sdk"

const { client, server } = await createOpencode()

// client: SDK 客户端实例
// server: 服务器进程引用
```

### 仅创建客户端（连接现有服务器）

```typescript
import { createOpencodeClient } from "@opencode-ai/sdk"

const client = createOpencodeClient({
  baseUrl: "http://localhost:4096"
})
```

## 配置选项

```typescript
const { client } = await createOpencode({
  hostname: "127.0.0.1",  // 服务器主机名
  port: 4096,             // 服务器端口
  timeout: 30000,         // 请求超时（毫秒）
  config: {               // 覆盖 opencode.json 配置
    model: "anthropic/claude-sonnet-4"
  }
})
```

## API 模块

SDK 按功能组织为多个模块：

### Global - 全局操作

```typescript
// 健康检查
const health = await client.global.health()
```

### App - 应用管理

```typescript
// 记录日志
await client.app.log({
  level: "info",
  message: "操作完成"
})

// 获取可用代理
const agents = await client.app.agents()
```

### Project - 项目管理

```typescript
// 获取当前项目
const project = await client.project.current()

// 列出所有项目
const projects = await client.project.list()
```

### Session - 会话管理

会话是核心 API，管理与 AI 的对话。

```typescript
// 创建会话
const session = await client.session.create({
  agent: "build"  // 可选，指定代理
})

// 发送消息（触发 AI 响应）
await client.session.chat(session.id, {
  content: "解释这个文件的作用",
  files: ["src/index.ts"]  // 可选，附加文件
})

// 注入上下文（不触发 AI 响应，适用于插件）
await client.session.inject(session.id, {
  content: "额外的上下文信息"
})

// 执行命令
await client.session.command(session.id, {
  command: "/test"
})

// 获取消息历史
const messages = await client.session.messages(session.id)

// 获取会话列表
const sessions = await client.session.list()

// 更新会话
await client.session.update(session.id, {
  title: "新标题"
})

// Fork 会话
const forked = await client.session.fork(session.id)

// 删除会话
await client.session.delete(session.id)

// 中止正在进行的操作
await client.session.abort(session.id)

// 处理权限请求
await client.session.permission(session.id, {
  granted: true
})
```

### Files - 文件操作

```typescript
// 搜索文本
const results = await client.files.search({
  query: "function handleClick",
  path: "src/"
})

// 查找文件
const files = await client.files.find({
  pattern: "**/*.ts"
})

// 符号查找
const symbols = await client.files.symbols({
  query: "handleClick"
})

// 读取文件
const content = await client.files.read({
  path: "src/index.ts"
})
```

### TUI - 界面控制

```typescript
// 显示提示
await client.tui.prompt({
  content: "请输入你的选择"
})

// 显示对话框
await client.tui.dialog({
  title: "确认",
  content: "确定要继续吗？",
  buttons: ["确定", "取消"]
})

// 显示通知
await client.tui.toast({
  message: "操作成功",
  level: "success"
})
```

### Auth - 认证管理

```typescript
// 列出已认证的提供商
const providers = await client.auth.list()

// 添加认证
await client.auth.add({
  provider: "anthropic",
  apiKey: "sk-..."
})
```

### Events - 事件流

```typescript
// 订阅服务器事件 (SSE)
const eventSource = client.events.subscribe()

eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data)
  console.log("事件:", data.type, data.payload)
}

// 关闭连接
eventSource.close()
```

## TypeScript 类型

所有类型都可以直接导入：

```typescript
import type {
  Session,
  Message,
  Part,
  Tool,
  Agent,
  Project,
  Provider
} from "@opencode-ai/sdk"
```

类型定义从服务器的 OpenAPI 规范自动生成。

## 完整示例

### 自动化代码审查

```typescript
import { createOpencode } from "@opencode-ai/sdk"

async function reviewCode(files: string[]) {
  const { client } = await createOpencode()

  // 创建会话
  const session = await client.session.create({
    agent: "build"
  })

  // 发送审查请求
  await client.session.chat(session.id, {
    content: `请审查以下文件的代码质量，检查潜在问题和改进建议：
${files.join("\n")}`,
    files
  })

  // 获取响应
  const messages = await client.session.messages(session.id)
  const lastMessage = messages[messages.length - 1]

  return lastMessage.content
}

// 使用
const review = await reviewCode([
  "src/components/Button.tsx",
  "src/utils/helpers.ts"
])
console.log(review)
```

### 实时事件监听

```typescript
import { createOpencodeClient } from "@opencode-ai/sdk"

const client = createOpencodeClient({
  baseUrl: "http://localhost:4096"
})

// 监听会话事件
const events = client.events.subscribe()

events.addEventListener("session.updated", (event) => {
  const { session } = JSON.parse(event.data)
  console.log("会话更新:", session.id)
})

events.addEventListener("message.updated", (event) => {
  const { message } = JSON.parse(event.data)
  console.log("新消息:", message.content)
})
```

### 批量文件处理

```typescript
import { createOpencode } from "@opencode-ai/sdk"

async function batchProcess(pattern: string, prompt: string) {
  const { client } = await createOpencode()

  // 查找文件
  const files = await client.files.find({ pattern })

  // 创建会话
  const session = await client.session.create({})

  // 处理每个文件
  for (const file of files) {
    await client.session.chat(session.id, {
      content: `${prompt}\n\n文件: ${file}`,
      files: [file]
    })
  }

  return await client.session.messages(session.id)
}
```

## 错误处理

```typescript
import { createOpencode, OpenCodeError } from "@opencode-ai/sdk"

try {
  const { client } = await createOpencode()
  await client.session.chat("invalid-id", { content: "test" })
} catch (error) {
  if (error instanceof OpenCodeError) {
    console.error("OpenCode 错误:", error.message)
    console.error("状态码:", error.status)
  } else {
    throw error
  }
}
```

## 下一步

- [插件开发](plugins.md) - 使用 SDK 开发插件
- [代理配置](agents.md) - 配置自定义代理
- [工具系统](tools.md) - 了解可用工具

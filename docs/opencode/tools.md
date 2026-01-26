# 工具系统

OpenCode 提供强大的内置工具，让 LLM 可以与代码库交互，同时支持自定义工具扩展。

## 内置工具

### bash - Shell 执行

在项目环境中执行 Shell 命令。

```
使用 bash 工具运行 npm install
```

**配置 Shell**:

默认使用 `$SHELL` 环境变量，或回退到 `/bin/bash`。

```json
{
  "shell": "/bin/zsh"
}
```

### read - 读取文件

获取文件内容。

```
读取 src/index.ts 的内容
```

### write - 写入文件

创建新文件或覆盖现有文件。

```
创建一个新的 utils.ts 文件
```

### edit - 编辑文件

使用精确字符串替换修改文件。

```
修改 handleClick 函数，添加错误处理
```

**工作原理**: 查找精确匹配的文本并替换为新内容。

### patch - 应用补丁

应用 patch 文件格式的修改。

```
应用这个 diff 到代码库
```

### glob - 文件匹配

通过模式匹配查找文件。

```
查找所有 .tsx 组件文件
```

**支持的模式**:
- `*.ts` - 当前目录的 .ts 文件
- `**/*.ts` - 递归所有 .ts 文件
- `src/**/*.{ts,tsx}` - src 下的 .ts 和 .tsx 文件

### grep - 正则搜索

使用正则表达式搜索文件内容。

```
搜索所有使用 useState 的地方
```

### list - 目录列表

显示目录内容。

```
列出 src/components 目录的文件
```

### lsp - 代码智能 (实验性)

访问语言服务器协议功能。

```
跳转到 handleClick 的定义
查找 UserService 的所有引用
```

**功能**:
- 跳转到定义
- 查找引用
- 符号查找
- 诊断信息

### webfetch - 网页获取

获取网页内容。

```
查看 React 文档中关于 useEffect 的说明
```

### todoread - 读取任务

读取现有任务列表。

### todowrite - 写入任务

创建和更新任务列表。

### question - 用户提问

在执行过程中向用户提问。

```
询问用户选择哪种实现方式
```

### skill - 技能加载

加载 SKILL.md 技能文件。

## 权限系统

工具通过权限系统控制访问级别：

| 权限 | 描述 |
|------|------|
| `allow` | 无限制使用 |
| `deny` | 完全禁止 |
| `ask` | 每次使用需要用户批准 |

### 配置权限

```json
{
  "permission": {
    "edit": "allow",
    "bash": "ask",
    "webfetch": "allow",
    "write": "ask"
  }
}
```

### Bash 命令细粒度控制

```json
{
  "permission": {
    "bash": {
      "npm *": "allow",
      "git *": "allow",
      "rm *": "deny",
      "*": "ask"
    }
  }
}
```

### 通配符

```json
{
  "permission": {
    "mymcp_*": "allow"
  }
}
```

## 自定义工具

### 创建工具

工具是 TypeScript/JavaScript 文件，放置在：
- `.opencode/tools/` - 项目级
- `~/.config/opencode/tools/` - 全局

**文件名即工具名**。

### 基础结构

```typescript
// .opencode/tools/greet.ts
import { tool } from "@opencode-ai/plugin"

export default tool({
  description: "向用户问好",
  args: {
    name: tool.schema.string().describe("用户名")
  },
  async execute(args, context) {
    return `你好，${args.name}！`
  }
})
```

### 带验证的工具

```typescript
import { tool } from "@opencode-ai/plugin"

export default tool({
  description: "计算两个数的和",
  args: {
    a: tool.schema.number().describe("第一个数"),
    b: tool.schema.number().describe("第二个数")
  },
  async execute({ a, b }) {
    return `${a} + ${b} = ${a + b}`
  }
})
```

### 多工具文件

```typescript
// .opencode/tools/math.ts
import { tool } from "@opencode-ai/plugin"

export const add = tool({
  description: "加法",
  args: {
    a: tool.schema.number(),
    b: tool.schema.number()
  },
  async execute({ a, b }) {
    return String(a + b)
  }
})

export const multiply = tool({
  description: "乘法",
  args: {
    a: tool.schema.number(),
    b: tool.schema.number()
  },
  async execute({ a, b }) {
    return String(a * b)
  }
})

// 创建工具: math_add, math_multiply
```

### 调用外部程序

```typescript
import { tool } from "@opencode-ai/plugin"
import { $ } from "bun"

export default tool({
  description: "运行 Python 脚本",
  args: {
    script: tool.schema.string().describe("脚本路径"),
    args: tool.schema.array(tool.schema.string()).optional()
  },
  async execute({ script, args = [] }) {
    const result = await $`python ${script} ${args.join(" ")}`
    return result.stdout.toString()
  }
})
```

### 上下文参数

```typescript
async execute(args, context) {
  // context.agent - 当前代理
  // context.sessionID - 会话 ID
  // context.messageID - 消息 ID
}
```

## Schema 选项

使用 `tool.schema`（基于 Zod）进行验证：

```typescript
tool.schema.string()              // 字符串
tool.schema.number()              // 数字
tool.schema.boolean()             // 布尔
tool.schema.array(schema)         // 数组
tool.schema.object({ key: schema }) // 对象
tool.schema.enum(["a", "b"])      // 枚举
tool.schema.optional()            // 可选
tool.schema.describe("说明")      // 添加描述
```

或直接使用 Zod：

```typescript
import { z } from "zod"

const args = {
  email: z.string().email(),
  age: z.number().min(0).max(150)
}
```

## 工具依赖

在 `.opencode/package.json` 中指定依赖：

```json
{
  "dependencies": {
    "lodash": "^4.17.21",
    "axios": "^1.6.0"
  }
}
```

## 禁用工具

### 全局禁用

```json
{
  "tools": {
    "webfetch": false
  }
}
```

### 按代理禁用

```json
{
  "agent": {
    "plan": {
      "tools": {
        "edit": false,
        "write": false,
        "bash": false
      }
    }
  }
}
```

## MCP 工具

通过 MCP 服务器添加外部工具，详见 [MCP 集成](mcp.md)。

```json
{
  "mcp": {
    "github": {
      "type": "local",
      "command": ["npx", "@modelcontextprotocol/server-github"]
    }
  }
}
```

## 最佳实践

1. **描述清晰**: 工具描述应明确说明功能和用途
2. **参数验证**: 使用 schema 验证所有输入
3. **错误处理**: 返回有意义的错误信息
4. **幂等性**: 尽量让工具操作可重复执行
5. **权限最小化**: 只请求必要的权限

## 下一步

- [插件开发](plugins.md) - 创建更复杂的扩展
- [MCP 集成](mcp.md) - 集成外部工具服务器
- [代理配置](agents.md) - 为代理配置工具

# 安装指南

## 系统要求

- macOS、Linux 或 Windows (WSL)
- Node.js 18+ (可选，用于 SDK)
- Git (推荐)

## 安装方法

### Homebrew (推荐)

```bash
# macOS / Linux
brew install opencode-ai/tap/opencode
```

### npm / pnpm / yarn

```bash
# npm
npm install -g opencode

# pnpm
pnpm add -g opencode

# yarn
yarn global add opencode
```

### 脚本安装

```bash
# 使用 curl
curl -fsSL https://opencode.ai/install | bash

# 使用 wget
wget -qO- https://opencode.ai/install | bash
```

### 从源码构建

```bash
git clone https://github.com/opencode-ai/opencode
cd opencode
go build -o opencode ./cmd/opencode
```

## SDK 安装

```bash
npm install @opencode-ai/sdk
```

## 初始配置

### 1. 连接提供商

首次使用需要连接 AI 提供商：

```bash
opencode
# 然后在 TUI 中输入 /connect
```

或直接运行：

```bash
opencode auth add anthropic
```

### 2. 配置文件

在项目根目录创建 `opencode.json`：

```json
{
  "model": "anthropic/claude-sonnet-4",
  "permission": {
    "bash": "ask",
    "edit": "allow",
    "webfetch": "allow"
  }
}
```

### 3. 验证安装

```bash
# 查看版本
opencode --version

# 查看已连接的提供商
opencode auth list

# 查看可用模型
opencode models
```

## 环境变量

| 变量 | 描述 | 默认值 |
|------|------|--------|
| `OPENCODE_SERVER_PASSWORD` | 服务器 HTTP Basic Auth 密码 | - |
| `OPENCODE_SERVER_USERNAME` | 服务器用户名 | `opencode` |
| `ANTHROPIC_API_KEY` | Anthropic API 密钥 | - |
| `OPENAI_API_KEY` | OpenAI API 密钥 | - |
| `SHELL` | bash 工具使用的 Shell | `/bin/bash` |

## 配置文件位置

| 类型 | 路径 |
|------|------|
| 全局配置 | `~/.config/opencode/opencode.json` |
| 项目配置 | `./opencode.json` 或 `./.opencode/opencode.json` |
| 认证信息 | `~/.local/share/opencode/auth.json` |
| 全局插件 | `~/.config/opencode/plugins/` |
| 项目插件 | `./.opencode/plugins/` |

## 服务器模式

### 独立服务器

```bash
opencode serve --port 4096 --hostname 127.0.0.1
```

### 带认证的服务器

```bash
export OPENCODE_SERVER_PASSWORD=your-password
opencode serve --port 4096
```

### 启用 CORS

```bash
opencode serve --cors http://localhost:3000 --cors https://myapp.com
```

### mDNS 发现

```bash
opencode serve --mdns
```

## 升级

```bash
# Homebrew
brew upgrade opencode

# npm
npm update -g opencode

# 脚本
curl -fsSL https://opencode.ai/install | bash
```

## 卸载

```bash
# Homebrew
brew uninstall opencode

# npm
npm uninstall -g opencode

# 清理配置
rm -rf ~/.config/opencode ~/.local/share/opencode
```

## 故障排除

### 常见问题

**问题**: `command not found: opencode`

**解决**: 确保安装路径在 `$PATH` 中：
```bash
export PATH="$PATH:$HOME/.local/bin"
```

**问题**: 无法连接到提供商

**解决**:
1. 检查 API 密钥是否正确
2. 检查网络连接
3. 运行 `opencode auth list` 验证认证状态

**问题**: 权限被拒绝

**解决**: 检查 `opencode.json` 中的权限配置，或使用 `--permission` 标志。

## 下一步

- [SDK 参考](sdk.md) - 学习程序化控制
- [提供商配置](providers.md) - 配置你的 AI 提供商
- [工具系统](tools.md) - 了解可用工具

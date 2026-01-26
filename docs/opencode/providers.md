# 提供商配置

OpenCode 支持 75+ LLM 提供商，通过 AI SDK 和 Models.dev 集成。

## 配置流程

所有提供商遵循标准两步配置：

1. 通过 `/connect` 命令添加 API 凭据
2. 在 `opencode.json` 中配置提供商

凭据存储在 `~/.local/share/opencode/auth.json`。

## 云端提供商

### Anthropic

```bash
# 连接
opencode auth add anthropic
```

```json
{
  "model": "anthropic/claude-sonnet-4",
  "provider": {
    "anthropic": {
      "options": {
        "baseURL": "https://api.anthropic.com/v1"
      }
    }
  }
}
```

支持的模型：
- `claude-opus-4-5` - 最强大的模型
- `claude-sonnet-4` - 平衡性能和成本
- `claude-haiku-3-5` - 快速响应

### OpenAI

```bash
opencode auth add openai
```

```json
{
  "model": "openai/gpt-5.1-codex",
  "provider": {
    "openai": {}
  }
}
```

支持 ChatGPT Plus/Pro 订阅或 API 密钥。

### Google Vertex AI

```bash
# 使用 gcloud CLI 认证
gcloud auth application-default login

# 或使用服务账号
opencode auth add google
```

```json
{
  "model": "google/gemini-3-pro",
  "provider": {
    "google": {
      "options": {
        "project": "your-project-id",
        "location": "us-central1"
      }
    }
  }
}
```

### Azure OpenAI

```json
{
  "provider": {
    "azure": {
      "options": {
        "resourceName": "your-resource",
        "apiVersion": "2024-02-01"
      },
      "models": {
        "gpt-4": {
          "name": "GPT-4",
          "deployment": "your-deployment-name"
        }
      }
    }
  }
}
```

### Amazon Bedrock

```json
{
  "provider": {
    "bedrock": {
      "options": {
        "region": "us-east-1",
        "profile": "default"
      }
    }
  }
}
```

支持 AWS profiles、access keys、bearer tokens 和 EKS IRSA 认证。

### DeepSeek

```bash
opencode auth add deepseek
```

```json
{
  "model": "deepseek/deepseek-coder"
}
```

### Groq

```bash
opencode auth add groq
```

```json
{
  "model": "groq/llama-3.3-70b-versatile"
}
```

### Together AI

```bash
opencode auth add together
```

```json
{
  "model": "together/meta-llama/Meta-Llama-3.1-405B-Instruct-Turbo"
}
```

### OpenRouter

```bash
opencode auth add openrouter
```

```json
{
  "model": "openrouter/anthropic/claude-sonnet-4"
}
```

### GitHub Copilot

使用设备流认证：

```bash
opencode auth add github-copilot
# 按提示在浏览器中完成认证
```

### GitLab Duo

```bash
opencode auth add gitlab
```

支持 OAuth 或个人访问令牌。自托管实例需要配置：

```json
{
  "provider": {
    "gitlab": {
      "options": {
        "baseURL": "https://gitlab.example.com",
        "aiGatewayURL": "https://ai-gateway.example.com"
      }
    }
  }
}
```

## 本地模型

### Ollama

```bash
# 安装 Ollama
curl -fsSL https://ollama.com/install.sh | sh

# 拉取模型
ollama pull codellama

# 连接
opencode auth add ollama
```

```json
{
  "model": "ollama/codellama",
  "provider": {
    "ollama": {
      "options": {
        "baseURL": "http://localhost:11434"
      }
    }
  }
}
```

### LM Studio

```json
{
  "provider": {
    "lmstudio": {
      "options": {
        "baseURL": "http://localhost:1234/v1"
      }
    }
  }
}
```

### llama.cpp

```json
{
  "provider": {
    "llamacpp": {
      "options": {
        "baseURL": "http://localhost:8080"
      }
    }
  }
}
```

## 自定义提供商

对于 OpenAI 兼容的提供商：

```bash
# 使用 /connect 命令选择 "Other"
# 输入唯一的提供商 ID
```

```json
{
  "provider": {
    "myprovider": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "My Provider",
      "options": {
        "baseURL": "https://api.example.com/v1",
        "headers": {
          "Authorization": "Bearer your-token"
        }
      },
      "models": {
        "custom-model": {
          "name": "Custom Model",
          "limit": {
            "context": 200000,
            "output": 65536
          }
        }
      }
    }
  }
}
```

## 代理网关

### Cloudflare AI Gateway

```json
{
  "provider": {
    "cloudflare": {
      "options": {
        "accountId": "your-account-id",
        "gatewayId": "your-gateway-id"
      }
    }
  }
}
```

### Vercel AI Gateway

```json
{
  "provider": {
    "vercel": {
      "options": {
        "baseURL": "https://ai-gateway.vercel.app/v1"
      }
    }
  }
}
```

### Helicone

```json
{
  "provider": {
    "anthropic": {
      "options": {
        "baseURL": "https://anthropic.helicone.ai/v1",
        "headers": {
          "Helicone-Auth": "Bearer your-key"
        }
      }
    }
  }
}
```

## 模型配置

### 默认模型

```json
{
  "model": "provider_id/model_id"
}
```

### 模型变体

```json
{
  "provider": {
    "anthropic": {
      "models": {
        "claude-sonnet-4": {
          "variants": {
            "high": {
              "thinking": {
                "budgetTokens": 10000
              }
            },
            "max": {
              "thinking": {
                "budgetTokens": 50000
              }
            }
          }
        }
      }
    }
  }
}
```

### 推理配置

```json
{
  "provider": {
    "openai": {
      "models": {
        "gpt-5.1-codex": {
          "reasoningEffort": "high",
          "textVerbosity": "normal"
        }
      }
    }
  }
}
```

## 命令参考

```bash
# 添加提供商凭据
opencode auth add <provider>

# 或使用 TUI
/connect

# 查看已认证的提供商
opencode auth list

# 查看可用模型
/models

# 切换模型
opencode --model provider/model
```

## 优先级顺序

模型选择优先级：

1. 命令行标志 (`--model` 或 `-m`)
2. 配置文件中的 `model` 键
3. 上次使用的模型
4. 按内部优先级的第一个模型

## 完整提供商列表

| 提供商 | 认证方式 | 特点 |
|--------|----------|------|
| 302.AI | API Key | 中国区优化 |
| Amazon Bedrock | AWS Auth | 企业级 |
| Anthropic | API Key / Pro | Claude 系列 |
| Azure OpenAI | Azure Auth | 企业合规 |
| Cerebras | API Key | 高速推理 |
| DeepSeek | API Key | 代码优化 |
| Fireworks AI | API Key | 开源模型 |
| GitHub Copilot | OAuth | VS Code 集成 |
| GitLab Duo | OAuth/PAT | GitLab 集成 |
| Google Vertex | GCP Auth | Gemini 系列 |
| Groq | API Key | 超低延迟 |
| Hugging Face | Token | 开源模型 |
| Moonshot AI | API Key | Kimi 系列 |
| Ollama | Local | 本地模型 |
| OpenAI | API Key / Plus | GPT 系列 |
| OpenRouter | API Key | 多模型聚合 |
| Together AI | API Key | 开源模型 |
| xAI | API Key | Grok 系列 |

## 下一步

- [模型配置](models.md) - 详细模型配置
- [代理配置](agents.md) - 为不同任务配置代理
- [SDK 参考](sdk.md) - 程序化访问

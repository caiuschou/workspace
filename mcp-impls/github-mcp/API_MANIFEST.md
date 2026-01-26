# GitHub MCP Server - API 清单

## 项目概述

GitHub MCP Server 是一个基于 Model Context Protocol (MCP) 的服务器实现，提供与 GitHub REST API 的集成能力。

- **版本**: 0.1.0
- **语言**: Rust
- **协议**: MCP (Model Context Protocol)
- **传输方式**: stdio, HTTP/SSE, WebSocket

---

## GitHub REST API 分类总览

| 大类 | 子类别 | 端点数量 | 优先级 | 说明 |
|------|--------|----------|--------|------|
| **核心仓库** | Repositories | 40+ | 高 | 仓库 CRUD、分支、标签、主题 |
| | Branches | 7 | 高 | 分支管理、保护规则、合并 |
| | Contents | 5 | 高 | 文件读写、目录、归档 |
| **代码协作** | Issues | 25+ | 高 | Issue、评论、标签、里程碑 |
| | Pull Requests | 15+ | 高 | PR、审查、评论、合并 |
| | Commits | 8 | 高 | 提交、评论、状态检查 |
| | Code Reviews | 6 | 中 | PR 审查流程管理 |
| **Git 数据** | Git Database | 8 | 中 | Blob、Tree、Commit、Tag、Ref |
| **组织管理** | Organizations | 20+ | 中 | 组织、成员、设置 |
| | Teams | 10+ | 中 | 团队、成员、权限 |
| | Collaborators | 5 | 中 | 协作者、邀请、权限 |
| **CI/CD** | Actions | 50+ | 中 | 工作流、运行、制品、密钥 |
| | Checks | 6 | 中 | 检查运行、套件 |
| | Deployments | 8 | 中 | 部署、环境、状态 |
| **发布管理** | Releases | 10+ | 中 | 版本发布、资源 |
| **搜索** | Search | 6 | 高 | 代码、仓库、Issue、用户 |
| **用户管理** | Users | 25+ | 低 | 用户、关注者、SSH/GPG 密钥 |
| **安全** | Code Scanning | 8 | 低 | 代码扫描告警 |
| | Secret Scanning | 5 | 低 | 密钥扫描告警 |
| | Security Advisories | 6 | 低 | 安全公告 |
| **分析** | Metrics/Stats | 7 | 低 | 统计、流量、社区概要 |
| **自动化** | Webhooks | 12+ | 中 | 事件通知 |
| | Activity | 5 | 低 | 事件、星标、订阅 |
| **基础设施** | Codespaces | 20+ | 低 | 云开发环境 |
| | Environments | 8 | 中 | 部署环境 |
| | Packages | 8 | 低 | 包管理 |
| | Pages | 6 | 低 | 静态站点 |
| **项目管理** | Projects V2 | 5 | 低 | 看板管理 |
| **社交** | Reactions | 12+ | 低 | 表情反应 |
| | Interactions | 3 | 低 | 交互限制 |
| **其他** | Gists | 10 | 低 | 代码片段 |
| | Gitignore | 2 | 低 | 模板 |
| | Licenses | 3 | 低 | 开源协议 |
| | Deploy Keys | 4 | 低 | 部署密钥 |
| | Emojis | 1 | 低 | 表情列表 |
| | Markdown | 2 | 低 | 渲染服务 |
| | Meta | 5 | 低 | API 元信息 |
| | Rate Limit | 1 | 中 | 速率限制查询 |
| | Billing | 3 | 低 | 账单信息 |
| **Wiki** | Wiki | * | 低 | 通过 Git 仓库操作（无专用 API） |

> **注意**: GitHub Wiki 没有专门的 REST API。Wiki 本质上是 Git 仓库（`{repo}.wiki.git`），可通过 Git API 或直接 Git 操作来管理。

**统计**: 共 30+ 大类，300+ API 端点

---

## 已实现的 MCP Tools

### 1. get_repository
获取 GitHub 仓库的基本信息。

| 属性 | 值 |
|------|-----|
| **名称** | `get_repository` |
| **描述** | Get information about a GitHub repository |
| **GitHub API** | `GET /repos/{owner}/{repo}` |

**参数**:
| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| `owner` | string | 是 | 仓库所有者（用户或组织） |
| `repo` | string | 是 | 仓库名称 |

**示例**:
```json
{
  "owner": "rust-lang",
  "repo": "rust"
}
```

---

### 2. list_issues
列出指定仓库的 Issue。

| 属性 | 值 |
|------|-----|
| **名称** | `list_issues` |
| **描述** | List issues in a GitHub repository |
| **GitHub API** | `GET /repos/{owner}/{repo}/issues` |

**参数**:
| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| `owner` | string | 是 | 仓库所有者 |
| `repo` | string | 是 | 仓库名称 |
| `state` | string | 否 | Issue 状态：`open`, `closed`, `all` |
| `limit` | number | 否 | 返回最大数量（默认：30） |

**示例**:
```json
{
  "owner": "rust-lang",
  "repo": "rust",
  "state": "open",
  "limit": 10
}
```

---

### 3. get_file
获取仓库中指定文件的内容。

| 属性 | 值 |
|------|-----|
| **名称** | `get_file` |
| **描述** | Get file content from a repository |
| **GitHub API** | `GET /repos/{owner}/{repo}/contents/{path}` |

**参数**:
| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| `owner` | string | 是 | 仓库所有者 |
| `repo` | string | 是 | 仓库名称 |
| `path` | string | 是 | 文件路径 |
| `branch` | string | 否 | 分支名称（默认：main） |

**示例**:
```json
{
  "owner": "rust-lang",
  "repo": "rust",
  "path": "README.md",
  "branch": "main"
}
```

---

### 4. create_issue
在仓库中创建新 Issue。

| 属性 | 值 |
|------|-----|
| **名称** | `create_issue` |
| **描述** | Create a new issue in a repository |
| **GitHub API** | `POST /repos/{owner}/{repo}/issues` |

**参数**:
| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| `owner` | string | 是 | 仓库所有者 |
| `repo` | string | 是 | 仓库名称 |
| `title` | string | 是 | Issue 标题 |
| `body` | string | 否 | Issue 描述 |
| `labels` | array | 否 | Issue 标签数组 |

**示例**:
```json
{
  "owner": "my-org",
  "repo": "my-repo",
  "title": "Bug report",
  "body": "Found a bug in the system",
  "labels": ["bug", "high-priority"]
}
```

---

## 计划实现的 API 清单

### 仓库操作 (Repositories)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出仓库 | GET | `/user/repos` | 中 |
| 列出组织仓库 | GET | `/orgs/{org}/repos` | 中 |
| 创建仓库 | POST | `/user/repos` | 中 |
| 更新仓库 | PATCH | `/repos/{owner}/{repo}` | 中 |
| 删除仓库 | DELETE | `/repos/{owner}/{repo}` | 低 |
| 列出分支 | GET | `/repos/{owner}/{repo}/branches` | 高 |
| 获取分支 | GET | `/repos/{owner}/{repo}/branches/{branch}` | 高 |
| 列出标签 | GET | `/repos/{owner}/{repo}/tags` | 中 |
| 列出提交 | GET | `/repos/{owner}/{repo}/commits` | 高 |

### Issue 操作 (Issues)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 获取单个 Issue | GET | `/repos/{owner}/{repo}/issues/{issue_number}` | 高 |
| 更新 Issue | PATCH | `/repos/{owner}/{repo}/issues/{issue_number}` | 高 |
| 关闭/重新打开 Issue | PATCH | `/repos/{owner}/{repo}/issues/{issue_number}` | 高 |
| 列出 Issue 评论 | GET | `/repos/{owner}/{repo}/issues/{issue_number}/comments` | 中 |
| 创建 Issue 评论 | POST | `/repos/{owner}/{repo}/issues/{issue_number}/comments` | 中 |
| 列出标签 | GET | `/repos/{owner}/{repo}/labels` | 中 |
| 创建标签 | POST | `/repos/{owner}/{repo}/labels` | 低 |

### Pull Request 操作 (Pull Requests)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出 Pull Requests | GET | `/repos/{owner}/{repo}/pulls` | 高 |
| 获取 Pull Request | GET | `/repos/{owner}/{repo}/pulls/{pull_number}` | 高 |
| 创建 Pull Request | POST | `/repos/{owner}/{repo}/pulls` | 高 |
| 更新 Pull Request | PATCH | `/repos/{owner}/{repo}/pulls/{pull_number}` | 中 |
| 列出 PR 评论 | GET | `/repos/{owner}/{repo}/pulls/{pull_number}/comments` | 中 |
| 创建 PR 评论 | POST | `/repos/{owner}/{repo}/pulls/{pull_number}/comments` | 中 |
| 请求审查 | POST | `/repos/{owner}/{repo}/pulls/{pull_number}/requested_reviewers` | 中 |
| 合并 Pull Request | PUT | `/repos/{owner}/{repo}/pulls/{pull_number}/merge` | 高 |

### 文件内容操作 (Contents)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 创建/更新文件 | PUT | `/repos/{owner}/{repo}/contents/{path}` | 高 |
| 删除文件 | DELETE | `/repos/{owner}/{repo}/contents/{path}` | 中 |
| 列出目录内容 | GET | `/repos/{owner}/{repo}/contents/{path}` | 中 |
| 获取归档 | GET | `/repos/{owner}/{repo}/{archive_format}` | 低 |

### Git 操作 (Git)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 获取引用 | GET | `/repos/{owner}/{repo}/git/ref/{ref}` | 中 |
| 创建引用 | POST | `/repos/{owner}/{repo}/git/refs` | 中 |
| 更新引用 | PATCH | `/repos/{owner}/{repo}/git/refs/{ref}` | 中 |
| 删除引用 | DELETE | `/repos/{owner}/{repo}/git/refs/{ref}` | 低 |
| 创建提交 | POST | `/repos/{owner}/{repo}/git/commits` | 中 |
| 创建树 | POST | `/repos/{owner}/{repo}/git/trees` | 低 |

### 用户操作 (Users)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 获取用户信息 | GET | `/users/{username}` | 中 |
| 获取认证用户 | GET | `/user` | 中 |
| 列出用户仓库 | GET | `/users/{username}/repos` | 低 |
| 列出关注者 | GET | `/users/{username}/followers` | 低 |

### 组织操作 (Organizations)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 获取组织信息 | GET | `/orgs/{org}` | 中 |
| 列出组织成员 | GET | `/orgs/{org}/members` | 中 |
| 列出组织仓库 | GET | `/orgs/{org}/repos` | 中 |
| 列出组织团队 | GET | `/orgs/{org}/teams` | 低 |

### 搜索操作 (Search)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 搜索仓库 | GET | `/search/repositories` | 高 |
| 搜索代码 | GET | `/search/code` | 高 |
| 搜索 Issue | GET | `/search/issues` | 中 |
| 搜索用户 | GET | `/search/users` | 低 |

### Release 操作 (Releases)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出 Release | GET | `/repos/{owner}/{repo}/releases` | 中 |
| 获取 Release | GET | `/repos/{owner}/{repo}/releases/{release_id}` | 中 |
| 创建 Release | POST | `/repos/{owner}/{repo}/releases` | 中 |
| 获取最新 Release | GET | `/repos/{owner}/{repo}/releases/latest` | 高 |

### Actions 操作 (Actions)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出工作流运行 | GET | `/repos/{owner}/{repo}/actions/runs` | 中 |
| 获取工作流运行 | GET | `/repos/{owner}/{repo}/actions/runs/{run_id}` | 中 |
| 重新运行工作流 | POST | `/repos/{owner}/{repo}/actions/runs/{run_id}/rerun` | 低 |

### Webhook 操作 (Webhooks)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出 Webhook | GET | `/repos/{owner}/{repo}/hooks` | 中 |
| 创建 Webhook | POST | `/repos/{owner}/{repo}/hooks` | 低 |
| 删除 Webhook | DELETE | `/repos/{owner}/{repo}/hooks/{hook_id}` | 低 |
| Ping Webhook | POST | `/repos/{owner}/{repo}/hooks/{hook_id}/pings` | 低 |

### 分支操作 (Branches)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出分支 | GET | `/repos/{owner}/{repo}/branches` | 高 |
| 获取分支 | GET | `/repos/{owner}/{repo}/branches/{branch}` | 高 |
| 获取分支保护 | GET | `/repos/{owner}/{repo}/branches/{branch}/protection` | 中 |
| 更新分支保护 | PUT/PATCH | `/repos/{owner}/{repo}/branches/{branch}/protection` | 中 |
| 删除分支保护 | DELETE | `/repos/{owner}/{repo}/branches/{branch}/protection` | 中 |
| 重命名分支 | POST | `/repos/{owner}/{repo}/branches/{branch}/rename` | 中 |
| 合并分支 | POST | `/repos/{owner}/{repo}/merges` | 高 |

### 提交操作 (Commits)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出提交 | GET | `/repos/{owner}/{repo}/commits` | 高 |
| 获取提交 | GET | `/repos/{owner}/{repo}/commits/{ref}` | 高 |
| 获取提交差异 | GET | `/repos/{owner}/{repo}/compare/{basehead}` | 中 |
| 列出提交评论 | GET | `/repos/{owner}/{repo}/commits/{commit_sha}/comments` | 中 |
| 创建提交评论 | POST | `/repos/{owner}/{repo}/commits/{commit_sha}/comments` | 中 |
| 创建提交状态 | POST | `/repos/{owner}/{repo}/statuses/{sha}` | 中 |
| 获取提交状态 | GET | `/repos/{owner}/{repo}/commits/{ref}/status` | 中 |

### 协作者操作 (Collaborators)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出协作者 | GET | `/repos/{owner}/{repo}/collaborators` | 中 |
| 添加协作者 | PUT | `/repos/{owner}/{repo}/collaborators/{username}` | 中 |
| 删除协作者 | DELETE | `/repos/{owner}/{repo}/collaborators/{username}` | 中 |
| 获取权限级别 | GET | `/repos/{owner}/{repo}/collaborators/{username}/permission` | 中 |
| 列出邀请 | GET | `/repos/{owner}/{repo}/invitations` | 低 |

### 部署操作 (Deployments)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出部署 | GET | `/repos/{owner}/{repo}/deployments` | 中 |
| 创建部署 | POST | `/repos/{owner}/{repo}/deployments` | 中 |
| 获取部署 | GET | `/repos/{owner}/{repo}/deployments/{deployment_id}` | 中 |
| 列出部署状态 | GET | `/repos/{owner}/{repo}/deployments/{deployment_id}/statuses` | 中 |
| 创建部署状态 | POST | `/repos/{owner}/{repo}/deployments/{deployment_id}/statuses` | 中 |
| 列出环境 | GET | `/repos/{owner}/{repo}/environments` | 中 |
| 获取/更新环境 | GET/PUT | `/repos/{owner}/{repo}/environments/{environment_name}` | 中 |
| 删除环境 | DELETE | `/repos/{owner}/{repo}/environments/{environment_name}` | 低 |

### 检查操作 (Checks)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 创建检查运行 | POST | `/repos/{owner}/{repo}/check-runs` | 中 |
| 获取检查运行 | GET | `/repos/{owner}/{repo}/check-runs/{check_run_id}` | 中 |
| 更新检查运行 | PATCH | `/repos/{owner}/{repo}/check-runs/{check_run_id}` | 中 |
| 列出检查运行 | GET | `/repos/{owner}/{repo}/commits/{ref}/check-runs` | 中 |
| 创建检查套件 | POST | `/repos/{owner}/{repo}/check-suites` | 低 |
| 获取检查套件 | GET | `/repos/{owner}/{repo}/check-suites/{check_suite_id}` | 低 |

### 活动操作 (Activity)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出仓库事件 | GET | `/repos/{owner}/{repo}/events` | 低 |
| 列出星标用户 | GET | `/repos/{owner}/{repo}/stargazers` | 低 |
| 列出订阅者 | GET | `/repos/{owner}/{repo}/subscribers` | 低 |
| 检查星标状态 | GET | `/user/starred/{owner}/{repo}` | 低 |
| 星标/取消星标 | PUT/DELETE | `/user/starred/{owner}/{repo}` | 低 |

### 指标操作 (Metrics)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 获取社区概要 | GET | `/repos/{owner}/{repo}/community/profile` | 低 |
| 获取代码频率 | GET | `/repos/{owner}/{repo}/stats/code_frequency` | 低 |
| 获取提交活动 | GET | `/repos/{owner}/{repo}/stats/commit_activity` | 低 |
| 获取贡献者 | GET | `/repos/{owner}/{repo}/stats/contributors` | 低 |
| 获取访问量 | GET | `/repos/{owner}/{repo}/traffic/views` | 低 |
| 获取克隆数 | GET | `/repos/{owner}/{repo}/traffic/clones` | 低 |
| 获取引用路径 | GET | `/repos/{owner}/{repo}/traffic/popular/paths` | 低 |

### 团队操作 (Teams)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出团队 | GET | `/orgs/{org}/teams` | 中 |
| 创建团队 | POST | `/orgs/{org}/teams` | 中 |
| 获取团队 | GET | `/orgs/{org}/teams/{team_slug}` | 中 |
| 更新团队 | PATCH | `/orgs/{org}/teams/{team_slug}` | 中 |
| 删除团队 | DELETE | `/orgs/{org}/teams/{team_slug}` | 低 |
| 列出团队成员 | GET | `/orgs/{org}/teams/{team_slug}/members` | 中 |
| 添加/删除成员 | PUT/DELETE | `/orgs/{org}/teams/{team_slug}/memberships/{username}` | 中 |
| 列出团队仓库 | GET | `/orgs/{org}/teams/{team_slug}/repos` | 中 |
| 添加/删除仓库 | PUT/DELETE | `/orgs/{org}/teams/{team_slug}/repos/{owner}/{repo}` | 中 |

### 标签和里程碑 (Labels & Milestones)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出标签 | GET | `/repos/{owner}/{repo}/labels` | 中 |
| 获取标签 | GET | `/repos/{owner}/{repo}/labels/{name}` | 中 |
| 创建标签 | POST | `/repos/{owner}/{repo}/labels` | 中 |
| 更新标签 | PATCH | `/repos/{owner}/{repo}/labels/{name}` | 中 |
| 删除标签 | DELETE | `/repos/{owner}/{repo}/labels/{name}` | 低 |
| 列出里程碑 | GET | `/repos/{owner}/{repo}/milestones` | 中 |
| 获取里程碑 | GET | `/repos/{owner}/{repo}/milestones/{milestone_number}` | 中 |
| 创建里程碑 | POST | `/repos/{owner}/{repo}/milestones` | 中 |
| 更新里程碑 | PATCH | `/repos/{owner}/{repo}/milestones/{milestone_number}` | 中 |
| 删除里程碑 | DELETE | `/repos/{owner}/{repo}/milestones/{milestone_number}` | 低 |

### 反应操作 (Reactions)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出 Issue 反应 | GET | `/repos/{owner}/{repo}/issues/{issue_number}/reactions` | 低 |
| 创建 Issue 反应 | POST | `/repos/{owner}/{repo}/issues/{issue_number}/reactions` | 低 |
| 删除反应 | DELETE | `/repos/{owner}/{repo}/issues/comments/{comment_id}/reactions/{reaction_id}` | 低 |
| 列出评论反应 | GET | `/repos/{owner}/{repo}/issues/comments/{comment_id}/reactions` | 低 |

### 项目操作 (Projects)
| 功能 | 方法 | 端点 | 优先级 |
|------|------|------|--------|
| 列出组织项目 V2 | GET | `/orgs/{org}/projectsV2` | 低 |
| 获取项目 V2 | GET | `/orgs/{org}/projectsV2/{project_number}` | 低 |
| 列出项目项 | GET | `/orgs/{org}/projectsV2/{project_number}/items` | 低 |
| 创建项目项 | POST | `/orgs/{org}/projectsV2/{project_number}/items` | 低 |
| 更新项目项 | PATCH | `/orgs/{org}/projectsV2/{project_number}/items/{item_id}` | 低 |

### 其他 API 类别

| 类别 | 端点示例 | 优先级 |
|------|----------|--------|
| **Gists** | `/gists`, `/gists/{gist_id}` | 低 |
| **Packages** | `/orgs/{org}/packages`, `/user/packages` | 低 |
| **Pages** | `/repos/{owner}/{repo}/pages` | 低 |
| **Codespaces** | `/user/codespaces`, `/repos/{owner}/{repo}/codespaces` | 低 |
| **Code Scanning** | `/repos/{owner}/{repo}/code-scanning/alerts` | 低 |
| **Secret Scanning** | `/repos/{owner}/{repo}/secret-scanning/alerts` | 低 |
| **Security Advisories** | `/repos/{owner}/{repo}/security-advisories` | 低 |
| **Licenses** | `/licenses`, `/repos/{owner}/{repo}/license` | 低 |
| **Gitignore** | `/gitignore/templates` | 低 |
| **Deploy Keys** | `/repos/{owner}/{repo}/keys` | 低 |
| **Interactions** | `/repos/{owner}/{repo}/interaction-limits` | 低 |
| **Rate Limit** | `/rate_limit` | 中 |
| **Markdown** | `/markdown` | 低 |
| **Emojis** | `/emojis` | 低 |
| **Meta** | `/`, `/meta` | 低 |
| **Billing** | `/orgs/{org}/settings/billing/*` | 低 |

---

## 配置说明

### 环境变量
| 变量 | 必需 | 描述 |
|------|------|------|
| `GITHUB_TOKEN` | 否 | GitHub Personal Access Token（推荐设置以获得更高速率限制） |

### GitHub API 版本
- **当前版本**: `2022-11-28`
- **Base URL**: `https://api.github.com`

### 速率限制
| 认证类型 | 速率限制 |
|----------|----------|
| 已认证 (PAT/App) | 5,000 请求/小时 |
| 未认证 | 60 请求/小时 |

---

## 错误处理

所有工具在 GitHub API 返回错误时会返回以下格式的错误响应：

```
GitHub API error ({status_code}): {error_message}
```

常见 HTTP 状态码：
- `401` - 未授权，检查 GITHUB_TOKEN
- `404` - 资源不存在
- `403` - 禁止访问（可能达到速率限制）
- `422` - 验证失败（参数错误）

---

## 相关文档

- [REST API 概览](./docs/rest-api-overview.md)
- [仓库 API](./docs/api-repositories.md)
- [Issue API](./docs/api-issues.md)
- [Pull Request API](./docs/api-pulls.md)
- [用户 API](./docs/api-users.md)
- [组织 API](./docs/api-organizations.md)
- [搜索 API](./docs/api-search.md)
- [Git API](./docs/api-git.md)
- [Actions API](./docs/api-actions.md)
- [Release API](./docs/api-releases.md)
- [Webhook API](./docs/api-webhooks.md)
- [认证文档](./docs/api-authentication.md)

---

## 开发路线图

### 阶段 1 - 核心功能（当前）
- [x] 基础 MCP 服务器框架
- [x] 仓库信息获取
- [x] Issue 列表和创建
- [x] 文件内容获取

### 阶段 2 - 常用功能
- [ ] Pull Request 完整操作
- [ ] Issue 评论和更新
- [ ] 文件创建和更新
- [ ] 分支操作

### 阶段 3 - 高级功能
- [ ] 搜索功能
- [ ] Git 操作
- [ ] Actions 工作流
- [ ] Release 管理

### 阶段 4 - 扩展功能
- [ ] Webhook 管理
- [ ] 组织和团队管理
- [ ] 检查和状态 API

---

## 许可证

详见项目根目录的 LICENSE 文件。

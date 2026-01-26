# GitLab MCP Server

A Model Context Protocol (MCP) server for GitLab, built with Rust.

## Features

### MCP Server
- 基于 MCP 协议，与 AI 助手（如 Claude）无缝集成
- 通过 stdio 进行通信
- 完整的 GitLab API 工具支持

### CLI Client
- **自动补全**: 支持命令、子命令、参数的自动补全
- **配置管理**: 支持多配置文件，方便切换不同 GitLab 实例
- **输出格式**: 支持 table、json、plain 等多种输出格式
- **别名支持**: 可为常用命令设置别名
- **交互模式**: 支持进入交互式 shell
- **批量操作**: 支持从文件读取批量执行命令

## 工具列表

| 模块 | 工具 | 说明 | 状态 |
|-----|------|-----|------|
| **项目管理** | `get_project` | 获取项目详情 | ✅ |
| | `list_projects` | 列出用户可访问的项目 | ✅ |
| | `create_project` | 创建新项目 | ✅ |
| | `get_project_members` | 获取项目成员列表 | ✅ |
| **Issue** | `list_issues` | 列出项目的 Issues | ✅ |
| | `get_issue` | 获取单个 Issue 详情 | ✅ |
| | `create_issue` | 创建新 Issue | 🟡 |
| | `update_issue` | 更新 Issue | ❌ |
| | `add_issue_note` | 添加 Issue 评论 | ❌ |
| **Merge Request** | `list_merge_requests` | 列出 MRs | ✅ |
| | `get_merge_request` | 获取 MR 详情 | ✅ |
| | `create_merge_request` | 创建 MR | 🟡 |
| | `merge_merge_request` | 合并 MR | ❌ |
| | `add_mr_note` | 添加 MR 评论 | ❌ |
| **Pipeline** | `list_pipelines` | 列出项目的 Pipelines | ✅ |
| | `get_pipeline` | 获取 Pipeline 详情 | 🟡 |
| | `get_pipeline_jobs` | 获取 Pipeline 的 Jobs | ❌ |
| | `get_job_log` | 获取 Job 日志 | ❌ |
| | `trigger_pipeline` | 手动触发 Pipeline | ❌ |
| **仓库文件** | `list_files` | 列出目录文件 | ✅ |
| | `get_file` | 获取文件内容 | ✅ |
| | `create_file` | 创建新文件 | ❌ |
| | `update_file` | 更新文件 | ❌ |
| | `delete_file` | 删除文件 | ❌ |
| **分支** | `list_branches` | 列出分支 | ✅ |
| | `get_branch` | 获取单个分支 | 🟡 |
| | `create_branch` | 创建分支 | 🟡 |
| | `delete_branch` | 删除分支 | ❌ |
| **提交** | `list_commits` | 列出提交 | ✅ |
| | `get_commit` | 获取单个提交 | ❌ |
| | `get_commit_diff` | 获取提交 diff | ❌ |
| | `cherry_pick_commit` | Cherry-pick 提交 | ❌ |
| | `revert_commit` | 回滚提交 | ❌ |
| **标签** | `list_tags` | 列出标签 | ❌ |
| | `create_tag` | 创建标签 | ❌ |
| | `delete_tag` | 删除标签 | ❌ |
| **里程碑** | `list_milestones` | 列出里程碑 | ❌ |
| | `create_milestone` | 创建里程碑 | ❌ |
| | `update_milestone` | 更新里程碑 | ❌ |
| **Wiki** | `list_wiki_pages` | 列出项目的 Wiki 页面 | ❌ |
| | `get_wiki_page` | 获取单个 Wiki 页面 | ❌ |
| | `create_wiki_page` | 创建 Wiki 页面 | ❌ |
| | `update_wiki_page` | 更新 Wiki 页面 | ❌ |
| | `delete_wiki_page` | 删除 Wiki 页面 | ❌ |
| **环境** | `list_environments` | 列出环境 | ❌ |
| | `create_environment` | 创建环境 | ❌ |
| | `stop_environment` | 停止环境 | ❌ |
| **发布** | `list_releases` | 列出发布 | ❌ |
| | `create_release` | 创建发布 | ❌ |
| | `update_release` | 更新发布 | ❌ |
| **用户** | `get_current_user` | 获取当前用户信息 | ❌ |
| | `list_users` | 列出用户 | ❌ |

**状态说明**:
- ✅ 已完成 (Server + CLI)
- 🟡 部分完成 (Server 已实现，CLI 待实现)
- ❌ 未实现

## Installation

```bash
cargo install --path .
```

## Configuration

Set environment variables:

```bash
export GITLAB_URL="https://gitlab.com"
export GITLAB_TOKEN="glpat-xxxxxxxxxxxx"
```

## Claude Desktop Configuration

```json
{
  "mcpServers": {
    "gitlab": {
      "command": "/path/to/gitlab-mcp-server",
      "env": {
        "GITLAB_URL": "https://gitlab.com",
        "GITLAB_TOKEN": "glpat-xxxxxxxxxxxx"
      }
    }
  }
}
```

## CLI Usage

```bash
# List projects
gitlab-mcp project list --search "my-project"

# Create an issue
gitlab-mcp issue create 123 --title "Fix bug" --labels "bug,high"

# List merge requests
gitlab-mcp mr list 123 --state opened

# Create a merge request
gitlab-mcp mr create 123 --source-branch "feature" --target-branch "main" --title "Add feature"

# List pipelines
gitlab-mcp pipeline list 123 --ref main

# List branches
gitlab-mcp branch list 123

# Create a branch
gitlab-mcp branch create 123 --name "feature/x" --from "main"
```

## Development

详见 [DEV.md](DEV.md)。

## Changelog

详见 [CHANGELOGS.md](CHANGELOGS.md)。

## License

MIT

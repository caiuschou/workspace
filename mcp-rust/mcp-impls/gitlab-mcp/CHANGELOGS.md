# 更新日志

所有重要变更都将记录在此文件中。

## [Unreleased]

### 计划中
- Issue: update_issue, add_issue_note, list_issue_notes
- Merge Request: update_merge_request, merge_merge_request, add_mr_note, list_mr_discussions
- Branch: delete_branch, delete_merged_branches
- Commit: get_commit, get_commit_diff, cherry_pick_commit, revert_commit
- Pipeline: get_pipeline_jobs, get_job_log, retry_job, trigger_pipeline
- 仓库文件: create_file, update_file, delete_file
- Tag、Milestone、Wiki、Environment、Release 工具
- User 工具
- MCP 资源实现

## [0.1.0] - 2025-01-21

### 新增
- **MCP Server** - 基于 MCP 协议的 GitLab 服务器
- **CLI Client** - 命令行工具 `gitlab-mcp`
- **项目管理工具**
  - get_project - 获取项目详情
  - list_projects - 列出用户可访问的项目
  - create_project - 创建新项目
  - get_project_members - 获取项目成员列表
- **Issue 工具**
  - list_issues - 列出项目的 Issues
  - get_issue - 获取单个 Issue 详情
  - create_issue - 创建新 Issue (Server)
- **Merge Request 工具**
  - list_merge_requests - 列出 MRs
  - get_merge_request - 获取 MR 详情
  - create_merge_request - 创建 MR (Server)
- **Pipeline 工具**
  - list_pipelines - 列出项目的 Pipelines
  - get_pipeline - 获取 Pipeline 详情 (Server)
- **仓库文件工具**
  - list_files - 列出目录文件
  - get_file - 获取文件内容
- **Branch 工具**
  - list_branches - 列出分支
  - get_branch - 获取单个分支 (Server)
  - create_branch - 创建分支 (Server)
- **Commit 工具**
  - list_commits - 列出提交
- **配置管理**
  - 环境变量配置 (GITLAB_URL, GITLAB_TOKEN)
  - 配置文件支持 (~/.config/gitlab-mcp/config.toml)
  - 配置状态查询和设置命令

### CLI 命令
- `gitlab-mcp config` - 配置管理命令
- `gitlab-mcp project` - 项目管理命令
- `gitlab-mcp issue` - Issue 管理命令
- `gitlab-mcp mr` - Merge Request 管理命令
- `gitlab-mcp pipeline` - Pipeline 管理命令
- `gitlab-mcp repo` - 仓库文件命令
- `gitlab-mcp branch` - 分支管理命令

### 特性
- 支持 table、json、plain 多种输出格式
- 支持 GitLab.com 和自托管 GitLab 实例
- MCP stdio 传输层
- 日志记录支持

### 文档
- README.md - 项目介绍和使用说明
- DEV.md - 开发指南
- docs/solution.md - 解决方案文档

---

## 版本说明

- **状态图标**: ✅ 已完成 | 🟡 部分完成 | ❌ 未实现
- **Server**: 指 MCP Server 端实现
- **CLI**: 指命令行工具实现

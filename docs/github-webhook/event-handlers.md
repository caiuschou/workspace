# 事件处理器

> [返回目录](README.md)

## 1. Ping 事件处理器

```rust
// src/webhooks/github/handlers/ping.rs

use async_trait::async_trait;
use serde::Deserialize;

use super::super::{
    dispatcher::WebhookHandler,
    error::WebhookError,
};

#[derive(Debug, Deserialize)]
struct PingEvent {
    zen: Option<String>,
    hook_id: Option<u64>,
}

pub struct PingHandler;

#[async_trait]
impl WebhookHandler for PingHandler {
    fn event_type(&self) -> &'static str {
        "ping"
    }

    async fn handle(&self, payload: &[u8]) -> Result<(), WebhookError> {
        let ping: PingEvent = serde_json::from_slice(payload)?;

        tracing::info!(
            hook_id = ?ping.hook_id,
            zen = ?ping.zen,
            "Received ping event"
        );

        // Ping 事件通常不需要特殊处理
        Ok(())
    }
}
```

## 2. Push 事件处理器

```rust
// src/webhooks/github/handlers/push.rs

use async_trait::async_trait;

use super::super::{
    dispatcher::WebhookHandler,
    error::WebhookError,
    models::{PushEvent, Repository},
};

pub struct PushHandler;

#[async_trait]
impl WebhookHandler for PushHandler {
    fn event_type(&self) -> &'static str {
        "push"
    }

    async fn handle(&self, payload: &[u8]) -> Result<(), WebhookError> {
        let event: PushEvent = serde_json::from_slice(payload)?;

        // 过滤分支
        if !event.git_ref.starts_with("refs/heads/") {
            tracing::debug!(ref = %event.git_ref, "Ignoring non-branch push");
            return Ok(());
        }

        let branch = event.git_ref.strip_prefix("refs/heads/").unwrap_or(&event.git_ref);

        tracing::info!(
            repository = %event.repository.full_name,
            branch = %branch,
            pusher = %event.pusher.name,
            commits = event.commits.len(),
            "Push event received"
        );

        // 异步处理：触发 CI、通知等
        tokio::spawn(async move {
            // TODO: 实现具体业务逻辑
            // - 触发 CI/CD
            // - 发送通知
            // - 更新数据库状态
            handle_push_event(event).await;
        });

        Ok(())
    }
}

async fn handle_push_event(event: PushEvent) {
    // 实现具体业务逻辑
}
```

## 3. Pull Request 事件处理器

```rust
// src/webhooks/github/handlers/pull_request.rs

use async_trait::async_trait;

use super::super::{
    dispatcher::WebhookHandler,
    error::WebhookError,
    models::{PullRequestAction, PullRequestEvent},
};

pub struct PullRequestHandler;

#[async_trait]
impl WebhookHandler for PullRequestHandler {
    fn event_type(&self) -> &'static str {
        "pull_request"
    }

    async fn handle(&self, payload: &[u8]) -> Result<(), WebhookError> {
        let event: PullRequestEvent = serde_json::from_slice(payload)?;

        match event.action {
            PullRequestAction::Opened => {
                tracing::info!(
                    repository = %event.repository.full_name,
                    pr = event.number,
                    title = %event.pull_request.title,
                    author = %event.sender.login,
                    "PR opened"
                );

                tokio::spawn(async move {
                    handle_pr_opened(event).await;
                });
            }
            PullRequestAction::Closed => {
                if event.pull_request.merged.unwrap_or(false) {
                    tracing::info!(
                        repository = %event.repository.full_name,
                        pr = event.number,
                        "PR merged"
                    );
                } else {
                    tracing::info!(
                        repository = %event.repository.full_name,
                        pr = event.number,
                        "PR closed without merge"
                    );
                }
            }
            PullRequestAction::Synchronize => {
                tracing::debug!(
                    repository = %event.repository.full_name,
                    pr = event.number,
                    "PR synchronized"
                );
            }
            _ => {
                tracing::debug!(
                    action = ?event.action,
                    pr = event.number,
                    "Unhandled PR action"
                );
            }
        }

        Ok(())
    }
}

async fn handle_pr_opened(event: PullRequestEvent) {
    // 实现具体业务逻辑
    // - 自动代码审查
    // - 添加标签
    // - 通知相关团队
}
```

## 4. Issues 事件处理器

```rust
// src/webhooks/github/handlers/issues.rs

use async_trait::async_trait;

use super::super::{
    dispatcher::WebhookHandler,
    error::WebhookError,
    models::{IssueAction, IssuesEvent},
};

pub struct IssuesHandler;

#[async_trait]
impl WebhookHandler for IssuesHandler {
    fn event_type(&self) -> &'static str {
        "issues"
    }

    async fn handle(&self, payload: &[u8]) -> Result<(), WebhookError> {
        let event: IssuesEvent = serde_json::from_slice(payload)?;

        match event.action {
            IssueAction::Opened => {
                tracing::info!(
                    repository = %event.repository.full_name,
                    issue = event.issue.number,
                    title = %event.issue.title,
                    author = %event.sender.login,
                    "Issue opened"
                );

                tokio::spawn(async move {
                    handle_issue_opened(event).await;
                });
            }
            IssueAction::Closed => {
                tracing::info!(
                    repository = %event.repository.full_name,
                    issue = event.issue.number,
                    "Issue closed"
                );

                tokio::spawn(async move {
                    handle_issue_closed(event).await;
                });
            }
            IssueAction::Labeled => {
                tracing::debug!(
                    repository = %event.repository.full_name,
                    issue = event.issue.number,
                    labels = ?event.issue.labels.iter().map(|l| &l.name).collect::<Vec<_>>(),
                    "Issue labeled"
                );

                // 检查是否需要特殊处理
                for label in &event.issue.labels {
                    match label.name.as_str() {
                        "bug" | "critical" | "urgent" => {
                            tokio::spawn(async move {
                                handle_urgent_issue(event.clone()).await;
                            });
                        }
                        _ => {}
                    }
                }
            }
            IssueAction::Reopened => {
                tracing::info!(
                    repository = %event.repository.full_name,
                    issue = event.issue.number,
                    "Issue reopened"
                );
            }
            IssueAction::Edited => {
                tracing::debug!(
                    repository = %event.repository.full_name,
                    issue = event.issue.number,
                    "Issue edited"
                );
            }
            IssueAction::Assigned | IssueAction::Unassigned => {
                tracing::info!(
                    repository = %event.repository.full_name,
                    issue = event.issue.number,
                    action = ?event.action,
                    "Issue assignment changed"
                );
            }
            _ => {
                tracing::debug!(
                    action = ?event.action,
                    issue = event.issue.number,
                    "Unhandled issue action"
                );
            }
        }

        Ok(())
    }
}

/// 处理新打开的 Issue
async fn handle_issue_opened(event: IssuesEvent) {
    // 1. 发送通知到团队频道
    send_issue_notification(&event).await;

    // 2. 添加欢迎评论或模板提示
    // 3. 根据标题/内容自动添加标签
    // 4. 通知被指派的用户
}

/// 处理关闭的 Issue
async fn handle_issue_closed(event: IssuesEvent) {
    // 1. 更新统计信息
    // 2. 发送关闭通知
    // 3. 清理相关资源
}

/// 处理紧急 Issue（带有 bug/critical/urgent 标签）
async fn handle_urgent_issue(event: IssuesEvent) {
    // 1. 发送高优先级通知（PagerDuty、钉钉、Slack 等）
    // 2. 通知值班人员
    // 3. 记录到监控系统
}

async fn send_issue_notification(event: &IssuesEvent) {
    // 实现通知逻辑
}
```

### Issue 处理常见场景

| Action | 典型处理逻辑 |
|--------|--------------|
| `opened` | 发送通知、自动标签、添加欢迎评论、自动分配 |
| `labeled` | `bug`/`critical` 标签触发高优先级告警 |
| `closed` | 更新统计、发送关闭通知、归档相关任务 |
| `reopened` | 通知相关人员、重新排队处理 |
| `assigned` | 通知被分配用户、更新看板 |
| `edited` | 检测关键信息变化（如标题、优先级） |

## 5. Issue Comment 事件处理器

> 与 `issues` 事件不同，`issue_comment` 事件在 Issue 或 PR 中有人**发表评论**时触发。

```rust
// src/webhooks/github/handlers/issue_comment.rs

use async_trait::async_trait;

use super::super::{
    dispatcher::WebhookHandler,
    error::WebhookError,
    models::{Issue, Repository, User},
};

#[derive(Debug, serde::Deserialize)]
pub struct IssueCommentEvent {
    pub action: String,  // "created", "edited", "deleted"
    pub issue: Issue,
    pub comment: Comment,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Debug, serde::Deserialize)]
pub struct Comment {
    pub id: u64,
    pub body: String,
    pub user: User,
    pub created_at: String,
    pub html_url: String,
    #[serde(default)]
    pub issue_url: String,
}

pub struct IssueCommentHandler;

#[async_trait]
impl WebhookHandler for IssueCommentHandler {
    fn event_type(&self) -> &'static str {
        "issue_comment"
    }

    async fn handle(&self, payload: &[u8]) -> Result<(), WebhookError> {
        let event: IssueCommentEvent = serde_json::from_slice(payload)?;

        // 判断是 Issue 评论还是 PR 评论
        let is_pr = event.issue_url.contains("/pulls/");
        let target = if is_pr { "PR" } else { "Issue" };

        match event.action.as_str() {
            "created" => {
                tracing::info!(
                    repository = %event.repository.full_name,
                    target = %target,
                    number = event.issue.number,
                    author = %event.comment.user.login,
                    "Comment created"
                );

                // 检测评论中的命令
                handle_comment_commands(event).await;
            }
            "edited" => {
                tracing::debug!("Comment edited");
            }
            "deleted" => {
                tracing::debug!("Comment deleted");
            }
            _ => {}
        }

        Ok(())
    }
}

/// 检测并处理评论命令
async fn handle_comment_commands(event: IssueCommentEvent) {
    let body = event.comment.body.trim().to_lowercase();

    // 匹配命令模式
    if let Some(user) = extract_arg(&body, "/assign ") {
        // /assign @username → 分配给指定用户
        assign_user(&event, &user).await;
    } else if body.contains("/assign") {
        // /assign → 分配给评论者
        assign_to_commenter(&event).await;
    } else if body.contains("/close") {
        // /close → 关闭 Issue
        close_issue(&event).await;
    } else if let Some(labels) = extract_arg(&body, "/label ") {
        // /label bug,urgent → 添加标签
        add_labels(&event, labels).await;
    } else if body.contains("/reopen") {
        // /reopen → 重新打开
        reopen_issue(&event).await;
    } else if body.contains("/help") {
        // /help → 显示可用命令
        post_help(&event).await;
    }
}

fn extract_arg(body: &str, prefix: &str) -> Option<String> {
    body.find(prefix)
        .and_then(|pos| body.get(pos + prefix.len()..))
        .map(|s| s.trim().to_string())
}

// 命令实现函数（需要调用 GitHub API）
async fn assign_user(event: &IssueCommentEvent, user: &str) {
    // 调用 GitHub API 分配用户
}

async fn assign_to_commenter(event: &IssueCommentEvent) {
    // 分配给评论者
}

async fn close_issue(event: &IssueCommentEvent) {
    // 调用 GitHub API 关闭 Issue
}

async fn add_labels(event: &IssueCommentEvent, labels: &str) {
    // 解析逗号分隔的标签并添加
}

async fn reopen_issue(event: &IssueCommentEvent) {
    // 重新打开 Issue
}

async fn post_help(event: &IssueCommentEvent) {
    // 回复可用命令列表
}
```

### 常见评论命令

| 命令 | 功能 |
|------|------|
| `/assign [@user]` | 分配 Issue（指定用户或自己） |
| `/unassign` | 取消分配 |
| `/close` | 关闭 Issue |
| `/reopen` | 重新打开 Issue |
| `/label bug,urgent` | 添加标签（逗号分隔） |
| `/unlabel duplicate` | 移除标签 |
| `/duplicate #123` | 标记为重复 Issue |
| `/help` | 显示可用命令 |

## 6. 处理器注册

```rust
// src/webhooks/github/handlers/mod.rs

mod ping;
mod push;
mod pull_request;
mod issues;
mod issue_comment;

use super::dispatcher::WebhookDispatcher;

pub fn create_dispatcher() -> WebhookDispatcher {
    let mut dispatcher = WebhookDispatcher::new();

    dispatcher
        .register(ping::PingHandler)
        .register(push::PushHandler)
        .register(pull_request::PullRequestHandler)
        .register(issues::IssuesHandler)
        .register(issue_comment::IssueCommentHandler);

    dispatcher
}
```

## 7. 支持的事件类型

| 事件 | 描述 | 处理器状态 |
|------|------|------------|
| `ping` | Webhook 配置测试 | ✅ 已实现 |
| `push` | 代码推送 | ✅ 已实现 |
| `pull_request` | PR 操作 | ✅ 已实现 |
| `issues` | Issue 操作 | ✅ 已实现 |
| `issue_comment` | Issue/PR 评论 | ✅ 已实现 |
| `release` | 发布操作 | ⚠️ 待实现 |
| `workflow_run` | CI/CD 状态 | ⚠️ 待实现 |
| `check_suite` | 检查套件 | ⚠️ 待实现 |
| `deployment` | 部署事件 | ⚠️ 待实现 |

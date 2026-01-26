# 数据模型

> [返回目录](README.md)

## 1. Webhook 请求头

```rust
// src/webhooks/github/models.rs

use serde::{Deserialize, Serialize};

/// GitHub Webhook 请求头
#[derive(Debug, Clone)]
pub struct GitHubWebhookHeaders {
    /// 事件类型 (X-GitHub-Event)
    pub event: String,
    /// 投递 ID (X-GitHub-Delivery) - 用于幂等性检查
    pub delivery_id: String,
    /// 签名 (X-Hub-Signature-256)
    pub signature: Option<String>,
}
```

## 2. 通用模型

```rust
/// 仓库信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub default_branch: String,
}

/// 用户信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: u64,
    pub login: String,
    pub avatar_url: String,
}
```

## 3. Action 枚举 (类型安全)

```rust
/// Pull Request 动作类型
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestAction {
    Opened,
    Closed,
    Reopened,
    Synchronize,
    Edited,
    Assigned,
    Unassigned,
    ReviewRequested,
    ReviewRequestRemoved,
    Labeled,
    Unlabeled,
    ReadyForReview,
    ConvertedToDraft,
    #[serde(other)]
    Unknown,
}

/// Issue 动作类型
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IssueAction {
    Opened,
    Closed,
    Reopened,
    Edited,
    Assigned,
    Unassigned,
    Labeled,
    Unlabeled,
    Pinned,
    Unpinned,
    Deleted,
    Transferred,
    #[serde(other)]
    Unknown,
}
```

## 4. Push 事件

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct PushEvent {
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub before: String,
    pub after: String,
    pub repository: Repository,
    pub pusher: Pusher,
    pub commits: Vec<Commit>,
    pub head_commit: Option<Commit>,
    pub forced: bool,
    pub deleted: bool,
    pub created: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub timestamp: String,
    pub author: CommitAuthor,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pusher {
    pub name: String,
    pub email: Option<String>,
}
```

## 5. Pull Request 事件

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestEvent {
    pub action: PullRequestAction,
    pub number: u64,
    pub pull_request: PullRequest,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub head: GitRef,
    pub base: GitRef,
    pub merged: Option<bool>,
    pub draft: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitRef {
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub sha: String,
}
```

## 6. Issues 事件

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct IssuesEvent {
    pub action: IssueAction,
    pub issue: Issue,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub labels: Vec<Label>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    pub id: u64,
    pub name: String,
    pub color: String,
}
```

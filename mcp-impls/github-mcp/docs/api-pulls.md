# Pull Requests API

## Overview

The Pull Requests API allows you to list, create, update, and manage pull requests on GitHub repositories.

### Base Endpoint
```
https://api.github.com/repos/{owner}/{repo}/pulls
```

---

## List pull requests

### Endpoint
```
GET /repos/{owner}/{repo}/pulls
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner` | string | Yes | The account owner |
| `repo` | string | Yes | The repository name |
| `state` | string | No | Filter by state: `open`, `closed`, or `all` (default: `open`) |
| `head` | string | No | Filter by head user or branch (format: `user:ref`) |
| `base` | string | No | Filter by base branch |
| `sort` | string | No | Sort by: `created`, `updated`, `popularity`, `long-running` (default: `created`) |
| `direction` | string | No | Direction: `asc` or `desc` (default: `desc`) |
| `per_page` | integer | No | Results per page (max 100, default 30) |
| `page` | integer | No | Page number (default 1) |

### Request Example
```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/pulls?state=open&base=main
```

### Response
```json
[
  {
    "id": 1,
    "node_id": "MDExOlB1bGxSZXF1ZXN0MQ==",
    "number": 1347,
    "state": "open",
    "locked": false,
    "title": "new-feature",
    "user": {
      "login": "octocat",
      "id": 1,
      "avatar_url": "https://github.com/images/error/octocat_happy.gif"
    },
    "body": "Please pull these awesome changes",
    "labels": [
      {
        "id": 208045946,
        "name": "enhancement",
        "color": "fc2929"
      }
    ],
    "milestone": {
      "id": 1,
      "number": 1,
      "state": "open",
      "title": "v1.0"
    },
    "head": {
      "label": "octocat:new-feature",
      "ref": "new-feature",
      "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e",
      "user": {
        "login": "octocat",
        "id": 1
      },
      "repo": {
        "id": 1296269,
        "name": "Hello-World",
        "full_name": "octocat/Hello-World"
      }
    },
    "base": {
      "label": "octocat:main",
      "ref": "main",
      "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e",
      "user": {
        "login": "octocat",
        "id": 1
      },
      "repo": {
        "id": 1296269,
        "name": "Hello-World",
        "full_name": "octocat/Hello-World"
      }
    },
    "mergeable": true,
    "mergeable_state": "clean",
    "merged": false,
    "merged_at": null,
    "merged_by": null,
    "merge_commit_sha": "e5bd3914e2e596debea16f433f57875b5b90bcd6",
    "comments": 10,
    "review_comments": 4,
    "commits": 3,
    "additions": 100,
    "deletions": 50,
    "changed_files": 5,
    "created_at": "2011-04-22T13:33:48Z",
    "updated_at": "2011-04-22T13:33:48Z",
    "closed_at": null,
    "html_url": "https://github.com/octocat/Hello-World/pull/1347",
    "draft": false
  }
]
```

---

## Get a pull request

### Endpoint
```
GET /repos/{owner}/{repo}/pulls/{pull_number}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner` | string | Yes | The account owner |
| `repo` | string | Yes | The repository name |
| `pull_number` | integer | Yes | The pull request number |

---

## Create a pull request

### Endpoint
```
POST /repos/{owner}/{repo}/pulls
```

### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | Yes | The title of the pull request |
| `head` | string | Yes | The name of the branch with your changes (format: `user:branch`) |
| `base` | string | Yes | The name of the branch to merge into |
| `body` | string | No | The contents of the pull request |
| `maintainer_can_modify` | boolean | No | Enable maintainers to modify (default: `true`) |
| `draft` | boolean | No | Create as draft (default: `false`) |
| `issue` | integer | No | The issue number to convert to a PR |

### Request Example
```bash
curl -L \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/pulls \
  -d '{
    "title": "Amazing new feature",
    "head": "octocat:new-feature",
    "base": "main",
    "body": "Please pull these awesome changes in!",
    "draft": false,
    "maintainer_can_modify": true
  }'
```

---

## Update a pull request

### Endpoint
```
PATCH /repos/{owner}/{repo}/pulls/{pull_number}
```

### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | No | The title of the pull request |
| `body` | string | No | The contents of the pull request |
| `state` | string | No | `open` or `closed` |
| `base` | string | No | The name of the base branch |
| `maintainer_can_modify` | boolean | No | Enable maintainers to modify |

---

## List commits in a pull request

### Endpoint
```
GET /repos/{owner}/{repo}/pulls/{pull_number}/commits
```

---

## List pull request files

### Endpoint
```
GET /repos/{owner}/{repo}/pulls/{pull_number}/files
```

### Response includes changed files with:
- `filename`
- `status` (`added`, `modified`, `removed`, `renamed`)
- `additions`
- `deletions`
- `changes`
- `patch` (diff)

---

## Merge a pull request

### Endpoint
```
PUT /repos/{owner}/{repo}/pulls/{pull_number}/merge
```

### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `commit_title` | string | No | Title for the automatic commit message |
| `commit_message` | string | No | Extra detail for the commit message |
| `sha` | string | No | SHA that the PR must be updated to before merging |
| `merge_method` | string | No | `merge`, `squash`, or `rebase` (default: `merge`) |

### Request Example
```bash
curl -L \
  -X PUT \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/pulls/PULL_NUMBER/merge \
  -d '{
    "commit_title": "Merged PR #123",
    "commit_message": "This is a nice change",
    "merge_method": "merge"
  }'
```

### Response (Success)
```json
{
  "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e",
  "merged": true,
  "message": "Pull Request successfully merged"
}
```

---

## Pull Request Reviews API

### List reviews
```
GET /repos/{owner}/{repo}/pulls/{pull_number}/reviews
```

### Get a review
```
GET /repos/{owner}/{repo}/pulls/{pull_number}/reviews/{review_id}
```

### Create a review
```
POST /repos/{owner}/{repo}/pulls/{pull_number}/reviews
```
```json
{
  "commit_id": "ecdd80bb57125d8ba3b1b912e0d9e52d3465cd13",
  "body": "This looks great!",
  "event": "APPROVE",
  "comments": [
    {
      "path": "file.md",
      "position": 5,
      "body": "Consider renaming this"
    }
  ]
}
```

| Event Value | Description |
|-------------|-------------|
| `APPROVE` | Approve the PR |
| `REQUEST_CHANGES` | Request changes |
| `COMMENT` | Comment without reviewing |
| `DISMISS` | Dismiss a review |

### Update a review
```
PUT /repos/{owner}/{repo}/pulls/{pull_number}/reviews/{review_id}
```

### Delete a review
```
DELETE /repos/{owner}/{repo}/pulls/{pull_number}/reviews/{review_id}
```

### List review comments
```
GET /repos/{owner}/{repo}/pulls/{pull_number}/comments
```

---

## Pull Request Comments API

### List review comments
```
GET /repos/{owner}/{repo}/pulls/{pull_number}/comments
```

### Create a review comment
```
POST /repos/{owner}/{repo}/pulls/{pull_number}/comments
```
```json
{
  "body": "Nice change!",
  "commit_id": "ecdd80bb57125d8ba3b1b912e0d9e52d3465cd13",
  "path": "file.md",
  "position": 5
}
```

### Get a review comment
```
GET /repos/{owner}/{repo}/pulls/comments/{comment_id}
```

### Update a review comment
```
PATCH /repos/{owner}/{repo}/pulls/comments/{comment_id}
```

### Delete a review comment
```
DELETE /repos/{owner}/{repo}/pulls/comments/{comment_id}
```

---

## Requested Reviewers API

### Request reviewers
```
POST /repos/{owner}/{repo}/pulls/{pull_number}/requested_reviewers
```
```json
{
  "reviewers": ["octocat", "hubot"],
  "team_reviewers": ["justice-league"]
}
```

### List requested reviewers
```
GET /repos/{owner}/{repo}/pulls/{pull_number}/requested_reviewers
```

### Remove requested reviewers
```
DELETE /repos/{owner}/{repo}/pulls/{pull_number}/requested_reviewers
```

---

## Related Endpoints

| Endpoint | Description |
|----------|-------------|
| `/repos/{owner}/{repo}/pulls/{pull_number}/update-branch` | Update PR branch with latest base branch |
| `/repos/{owner}/{repo}/pulls/{pull_number}/comments/{comment_id}/replies` | Create reply to review comment |

---

## Merge Methods

| Method | Description |
|--------|-------------|
| `merge` | Create a merge commit |
| `squash` | Squash all commits into one |
| `rebase` | Rebase commits onto base branch |

---

## Permissions

| Action | Required Permission |
|--------|---------------------|
| Read PRs | `Pull requests` repository permission (read) |
| Create/update/delete | `Pull requests` repository permission (write) |
| Merge | `Pull requests` repository permission (write) |

---

## Reference

- [Official Documentation](https://docs.github.com/en/rest/pulls/pulls)
- API Version: 2022-11-28

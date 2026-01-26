# Issues API

## Overview

The Issues API allows you to manage issues on GitHub repositories.

### Base Endpoint
```
https://api.github.com/repos/{owner}/{repo}/issues
```

---

## List repository issues

### Endpoint
```
GET /repos/{owner}/{repo}/issues
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner` | string | Yes | The account owner |
| `repo` | string | Yes | The repository name |
| `milestone` | string | No | Filter by milestone: `*`, `none`, or milestone number |
| `state` | string | No | Filter by state: `open`, `closed`, or `all` (default: `open`) |
| `assignee` | string | No | Filter by assignee: `*`, `none`, or username |
| `creator` | string | No | Filter by creator username |
| `mentioned` | string | No | Filter by mentioned user |
| `labels` | string | No | Comma-separated label names |
| `sort` | string | No | Sort by: `created`, `updated`, `comments` (default: `created`) |
| `direction` | string | No | Direction: `asc` or `desc` (default: `desc`) |
| `since` | string | No | Only show issues updated at or after this time (ISO 8601) |
| `per_page` | integer | No | Results per page (max 100, default 30) |
| `page` | integer | No | Page number (default 1) |

### Request Example
```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/issues?state=open&per_page=30
```

### Response
```json
[
  {
    "id": 1,
    "node_id": "MDU6SXNzdWUx",
    "url": "https://api.github.com/repos/octocat/Hello-World/issues/1347",
    "repository_url": "https://api.github.com/repos/octocat/Hello-World",
    "number": 1347,
    "state": "open",
    "title": "Found a bug",
    "body": "I'm having a problem with this.",
    "user": {
      "login": "octocat",
      "id": 1,
      "avatar_url": "https://github.com/images/error/octocat_happy.gif"
    },
    "labels": [
      {
        "id": 208045946,
        "name": "bug",
        "color": "fc2929",
        "default": true
      }
    ],
    "assignee": {
      "login": "octocat",
      "id": 1
    },
    "milestone": {
      "id": 1,
      "number": 1,
      "state": "open",
      "title": "v1.0"
    },
    "comments": 4,
    "created_at": "2011-04-22T13:33:48Z",
    "updated_at": "2011-04-22T13:33:48Z",
    "closed_at": null,
    "html_url": "https://github.com/octocat/Hello-World/issues/1347"
  }
]
```

---

## Get an issue

### Endpoint
```
GET /repos/{owner}/{repo}/issues/{issue_number}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner` | string | Yes | The account owner |
| `repo` | string | Yes | The repository name |
| `issue_number` | integer | Yes | The issue number |

---

## Create an issue

### Endpoint
```
POST /repos/{owner}/{repo}/issues
```

### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | Yes | The title of the issue |
| `body` | string | No | The contents of the issue |
| `assignee` | string | No | Login for the user to assign |
| `milestone` | integer | No | Milestone number |
| `labels` | array | No | Array of label names |
| `assignees` | array | No | Array of logins for users to assign |

### Request Example
```bash
curl -L \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/issues \
  -d '{
    "title": "Found a bug",
    "body": "I'\''m having a problem with this.",
    "labels": ["bug", "help wanted"],
    "assignees": ["octocat"]
  }'
```

---

## Update an issue

### Endpoint
```
PATCH /repos/{owner}/{repo}/issues/{issue_number}
```

### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | No | The title of the issue |
| `body` | string | No | The contents of the issue |
| `state` | string | No | `open` or `closed` |
| `state_reason` | string | No | `completed` or `not_planned` (when closing) |
| `assignee` | string | No | Login for the user to assign |
| `milestone` | integer/`null` | No | Milestone number or null to clear |
| `labels` | array | No | Array of label names |
| `assignees` | array | No | Array of logins for users to assign |

---

## Add labels to an issue

### Endpoint
```
POST /repos/{owner}/{repo}/issues/{issue_number}/labels
```

### Request Body
```json
{
  "labels": ["bug", "enhancement"]
}
```

---

## Set labels for an issue

### Endpoint
```
PUT /repos/{owner}/{repo}/issues/{issue_number}/labels
```

Replaces all existing labels with the provided labels.

---

## Remove a label from an issue

### Endpoint
```
DELETE /repos/{owner}/{repo}/issues/{issue_number}/labels/{name}
```

---

## Remove all labels from an issue

### Endpoint
```
DELETE /repos/{owner}/{repo}/issues/{issue_number}/labels
```

---

## List issue comments

### Endpoint
```
GET /repos/{owner}/{repo}/issues/{issue_number}/comments
```

---

## Create an issue comment

### Endpoint
```
POST /repos/{owner}/{repo}/issues/{issue_number}/comments
```

### Request Body
```json
{
  "body": "Me too!"
}
```

---

## Get an issue comment

### Endpoint
```
GET /repos/{owner}/{repo}/issues/comments/{comment_id}
```

---

## Update an issue comment

### Endpoint
```
PATCH /repos/{owner}/{repo}/issues/comments/{comment_id}
```

### Request Body
```json
{
  "body": "Updated comment"
}
```

---

## Delete an issue comment

### Endpoint
```
DELETE /repos/{owner}/{repo}/issues/comments/{comment_id}
```

---

## List issue events

### Endpoint
```
GET /repos/{owner}/{repo}/issues/{issue_number}/events
```

Returns timeline events for an issue.

---

## List assignees

### Endpoint
```
GET /repos/{owner}/{repo}/assignees
```

Lists available assignees (users with push access).

---

## Check if a user can be assigned

### Endpoint
```
GET /repos/{owner}/{repo}/assignees/{assignee}
```

Returns 204 if user can be assigned, 404 otherwise.

---

## Labels API

### List repository labels
```
GET /repos/{owner}/{repo}/labels
```

### Get a label
```
GET /repos/{owner}/{repo}/labels/{name}
```

### Create a label
```
POST /repos/{owner}/{repo}/labels
```
```json
{
  "name": "bug",
  "color": "fc2929",
  "description": "Something isn't working"
}
```

### Update a label
```
PATCH /repos/{owner}/{repo}/labels/{name}
```

### Delete a label
```
DELETE /repos/{owner}/{repo}/labels/{name}
```

---

## Milestones API

### List milestones
```
GET /repos/{owner}/{repo}/milestones
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `state` | string | `open`, `closed`, `all` (default: `open`) |
| `sort` | string | `due_on` or `completeness` (default: `due_on`) |
| `direction` | string | `asc` or `desc` (default: `asc`) |

### Get a milestone
```
GET /repos/{owner}/{repo}/milestones/{milestone_number}
```

### Create a milestone
```
POST /repos/{owner}/{repo}/milestones
```
```json
{
  "title": "v1.0",
  "state": "open",
  "description": "Tracking milestone for version 1.0",
  "due_on": "2012-10-09T23:39:01Z"
}
```

### Update a milestone
```
PATCH /repos/{owner}/{repo}/milestones/{milestone_number}
```

### Delete a milestone
```
DELETE /repos/{owner}/{repo}/milestones/{milestone_number}
```

---

## Related Endpoints

| Endpoint | Description |
|----------|-------------|
| `/repos/{owner}/{repo}/issues/{issue_number}/timeline` | Full timeline of events |
| `/repos/{owner}/{repo}/issues/{issue_number}/reactions` | List reactions |
| `/repos/{owner}/{repo}/issues/{issue_number}/comments/{comment_id}/reactions` | Comment reactions |
| `/repos/{owner}/{repo}/issues/{issue_number}/lock` | Lock conversation |
| `/repos/{owner}/{repo}/issues/{issue_number}/assignees` | Manage assignees |

---

## Permissions

| Action | Required Permission |
|--------|---------------------|
| Read issues | `Issues` repository permission (read) |
| Create/update/delete | `Issues` repository permission (write) |

---

## Reference

- [Official Documentation](https://docs.github.com/en/rest/issues/issues)
- API Version: 2022-11-28

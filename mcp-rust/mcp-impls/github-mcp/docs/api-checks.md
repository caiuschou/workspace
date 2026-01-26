# Checks API

## Overview

The Checks API allows you to create, update, and query check runs and check suites. Checks are used by continuous integration (CI) systems to test code changes.

### Base Endpoint
```
https://api.github.com/repos/{owner}/{repo}
```

---

## Check Runs API

### List check runs for a Git reference

#### Endpoint
```
GET /repos/{owner}/{repo}/commits/{ref}/check-runs
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `check_name` | string | No | Filters checks by name |
| `status` | string | No | Filters by status: `queued`, `in_progress`, `completed` |
| `app_id` | integer | No | Filters by GitHub App ID |
| `per_page` | integer | No | Results per page (max 100, default 30) |
| `page` | integer | No | Page number (default 1) |

---

### Get a check run

#### Endpoint
```
GET /repos/{owner}/{repo}/check-runs/{check_run_id}
```

---

### Create a check run

#### Endpoint
```
POST /repos/{owner}/{repo}/check-runs
```

#### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | The name of the check |
| `head_sha` | string | Yes | The SHA of the commit |
| `details_url` | string | No | URL for full details |
| `external_id` | string | No | A reference for the run on your system |
| `status` | string | No | `queued`, `in_progress`, or `completed` (default: `queued`) |
| `conclusion` | string | No | Required if status is `completed`: `success`, `failure`, `neutral`, `cancelled`, `timed_out`, `action_required` |
| `started_at` | string | No | ISO 8601 timestamp |
| `completed_at` | string | No | ISO 8601 timestamp |
| `output` | object | No | Check output |
| `actions` | array | No | Possible actions |

#### Output Object
```json
{
  "title": "Test summary",
  "summary": "Test summary text",
  "text": "Detailed test output",
  "annotations": [
    {
      "path": "README.md",
      "start_line": 2,
      "end_line": 2,
      "annotation_level": "notice",
      "message": "Check this line",
      "title": "Title",
      "raw_details": "Raw details"
    }
  ],
  "images": [
    {
      "path": "screenshot.png",
      "alt": "Screenshot",
      "caption": "Test result"
    }
  ]
}
```

#### Request Example
```bash
curl -L \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/check-runs \
  -d '{
    "name": "my-check",
    "head_sha": "aa218f56b14c9653891f9e74264a383fa43fefbd",
    "status": "completed",
    "conclusion": "success",
    "output": {
      "title": "Test summary",
      "summary": "Test passed"
    }
  }'
```

---

### Update a check run

#### Endpoint
```
PATCH /repos/{owner}/{repo}/check-runs/{check_run_id}
```

#### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | No | The name of the check |
| `details_url` | string | No | URL for full details |
| `external_id` | string | No | Reference for the run |
| `status` | string | No | `queued`, `in_progress`, or `completed` |
| `conclusion` | string | No | Final status when completed |
| `started_at` | string | No | ISO 8601 timestamp |
| `completed_at` | string | No | ISO 8601 timestamp |
| `output` | object | No | Check output |

---

### List check run annotations

#### Endpoint
```
GET /repos/{owner}/{repo}/check-runs/{check_run_id}/annotations
```

---

### Rerequest a check run

#### Endpoint
```
POST /repos/{owner}/{repo}/check-runs/{check_run_id}/rerequest
```

Triggers GitHub to rerequest the check run from the GitHub App.

---

## Check Suites API

### List check suites for a Git reference

#### Endpoint
```
GET /repos/{owner}/{repo}/commits/{ref}/check-suites
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `app_id` | integer | No | Filters by GitHub App ID |
| `check_name` | string | No | Filters checks by name |

---

### Get a check suite

#### Endpoint
```
GET /repos/{owner}/{repo}/check-suites/{check_suite_id}
```

---

### Create a check suite

#### Endpoint
```
POST /repos/{owner}/{repo}/check-suites
```

#### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `head_sha` | string | Yes | The SHA of the commit |
| `head_branch` | string | No | The branch name |

---

### Rerequest a check suite

#### Endpoint
```
POST /repos/{owner}/{repo}/check-suites/{check_suite_id}/rerequest
```

---

### List check runs in a check suite

#### Endpoint
```
GET /repos/{owner}/{repo}/check-suites/{check_suite_id}/check-runs
```

---

## Check Run Annotations

Annotations are comments you can add to specific lines of code to provide feedback.

### Annotation Levels

| Level | Description |
|-------|-------------|
| `notice` | Information that doesn't require attention |
| `warning` | Something that might require attention |
| `failure` | Something that requires attention |

### Annotation Object

```json
{
  "path": "src/file.js",
  "start_line": 10,
  "end_line": 15,
  "start_column": 5,
  "end_column": 10,
  "annotation_level": "warning",
  "message": "Unused variable",
  "title": "Lint warning",
  "raw_details": "Variable 'x' is never used"
}
```

---

## Permissions

| Action | Required Permission |
|--------|---------------------|
| Read checks | Contents (read) or Checks (read) |
| Create/update checks | Checks (write) |
| Re-request checks | Checks (write) |

---

## Check Run Status

| Status | Description |
|--------|-------------|
| `queued` | Check is queued |
| `in_progress` | Check is running |
| `completed` | Check has finished |

## Check Conclusion

| Conclusion | Description |
|------------|-------------|
| `success` | Check passed |
| `failure` | Check failed |
| `neutral` | Check neither passed nor failed |
| `cancelled` | Check was cancelled |
| `timed_out` | Check timed out |
| `action_required` | Action required from user |

---

## Reference

- API Version: 2022-11-28
- Endpoints listed in [api-endpoints.md](api-endpoints.md)

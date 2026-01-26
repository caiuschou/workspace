# Actions API

## Overview

The Actions API allows you to manage GitHub Actions workflows, runs, artifacts, cache, secrets, variables, and self-hosted runners.

### Base Endpoint
```
https://api.github.com/repos/{owner}/{repo}/actions
```

---

## Workflow Runs

### List workflow runs

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/runs
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `per_page` | integer | No | Results per page (max 100, default 30) |
| `page` | integer | No | Page number |
| `status` | string | No | Filter by status: `queued`, `in_progress`, `completed`, `waiting`, `pending`, `requested`, `failure` |
| `actor` | string | No | Filter by actor |
| `branch` | string | No | Filter by branch name |
| `event` | string | No | Filter by event: `push`, `pull_request`, etc. |
| `exclude_pull_requests` | boolean | No | Exclude pull request events |

---

### Get a workflow run

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/runs/{run_id}
```

---

### Cancel a workflow run

#### Endpoint
```
POST /repos/{owner}/{repo}/actions/runs/{run_id}/cancel
```

---

### Force cancel a workflow run

#### Endpoint
```
POST /repos/{owner}/{repo}/actions/runs/{run_id}/force-cancel
```

---

### Rerun a workflow run

#### Endpoint
```
POST /repos/{owner}/{repo}/actions/runs/{run_id}/rerun
```

---

### Rerun failed jobs in a workflow run

#### Endpoint
```
POST /repos/{owner}/{repo}/actions/runs/{run_id}/rerun-failed-jobs
```

---

### Get workflow run usage

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/runs/{run_id}/timing
```

---

### List workflow run jobs

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/runs/{run_id}/jobs
```

### Get a job for a workflow run

#### endpoint
```
GET /repos/{owner}/{repo}/actions/jobs/{job_id}
```

### Get job log for a workflow run

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/jobs/{job_id}/logs
```

### Download job log for a workflow run

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/jobs/{job_id}/logs
```

Accept: `application/vnd.github.v3+json`

---

## Workflow Jobs

### Rerun a job

#### Endpoint
```
POST /repos/{owner}/{repo}/actions/jobs/{job_id}/rerun
```

---

## Workflows

### List repository workflows

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/workflows
```

### Get a workflow

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/workflows/{workflow_id}
```

### Enable a workflow

#### Endpoint
```
PUT /repos/{owner}/{repo}/actions/workflows/{workflow_id}/enable
```

### Disable a workflow

#### Endpoint
```
PUT /repos/{owner}/{repo}/actions/workflows/{workflow_id}/disable
```

### Create a workflow dispatch event

#### Endpoint
```
POST /repos/{owner}/{repo}/actions/workflows/{workflow_id}/dispatches
```

### List workflow runs

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/workflows/{workflow_id}/runs
```

### Get workflow usage

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/workflows/{workflow_id}/timing
```

---

## Artifacts

### List artifacts

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/artifacts
```

### List artifacts for a workflow run

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/runs/{run_id}/artifacts
```

### Get an artifact

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/artifacts/{artifact_id}
```

### Delete an artifact

#### Endpoint
```
DELETE /repos/{owner}/{repo}/actions/artifacts/{artifact_id}
```

### Download an artifact

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/artifacts/{artifact_id}/{archive_format}
```

`archive_format` can be `zip`.

---

## Cache

### Get GitHub Actions cache usage for a repository

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/cache/usage
```

### List cache keys for a repository

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/caches
```

### Delete a cache

#### Endpoint
```
DELETE /repos/{owner}/{repo}/actions/caches/{cache_id}
```

### Delete caches

#### Endpoint
```
DELETE /repos/{owner}/{repo}/actions/caches
```

---

## Secrets

### List repository secrets

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/secrets
```

### Get a secret

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/secrets/{secret_name}
```

### Create or update a secret

#### Endpoint
```
PUT /repos/{owner}/{repo}/actions/secrets/{secret_name}
```

#### Request Body
```json
{
  "encrypted_value": "c3VwZXIzcnkgZGF0YQ=",
  "key_id": "2"
}
```

The `encrypted_value` must be encrypted using the public key. Get the public key first:
```
GET /repos/{owner}/{repo}/actions/secrets/public-key
```

### Delete a secret

#### endpoint
```
DELETE /repos/{owner}/{repo}/actions/secrets/{secret_name}
```

---

## Environment Secrets

### List environment secrets

#### Endpoint
```
GET /repositories/{repository_id}/environments/{environment_name}/secrets
```

### Get an environment secret

#### Endpoint
```
GET /repositories/{repository_id}/environments/{environment_name}/secrets/{secret_name}
```

### Create or update an environment secret

#### Endpoint
```
PUT /repositories/{repository_id}/environments/{environment_name}/secrets/{secret_name}
```

### Delete an environment secret

#### Endpoint
```
DELETE /repositories/{repository_id}/environments/{environment_name}/secrets/{secret_name}
```

---

## Variables

### List repository variables

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/variables
```

### Get a variable

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/variables/{name}
```

### Create a variable

#### Endpoint
```
POST /repos/{owner}/{repo}/actions/variables
```

#### Request Body
```json
{
  "name": "MY_VAR",
  "value": "my_value"
}
```

### Update a variable

#### Endpoint
```
PATCH /repos/{owner}/{repo}/actions/variables/{name}
```

### Delete a variable

#### Endpoint
```
DELETE /repos/{owner}/{repo}/actions/variables/{name}
```

### List environment variables

#### Endpoint
```
GET /repositories/{repository_id}/environments/{environment_name}/variables
```

---

## Self-Hosted Runners

### List self-hosted runners

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/runners
```

### Get a self-hosted runner

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/runners/{runner_id}
```

### Delete a self-hosted runner

#### Endpoint
```
DELETE /repos/{owner}/{repo}/actions/runners/{runner_id}
```

### Create a registration token for a repository

#### Endpoint
```
POST /repos/{owner}/{repo}/actions/runners/registration-token
```

### List runner applications

#### Endpoint
```
GET /repos/{owner}/{repo}/actions/runners/downloads
```

---

## Permissions

| Action | Required Permission |
|--------|---------------------|
| Read workflows/runs | Contents (read) |
| Manage workflows | Contents (write) |
| Manage artifacts | Contents (read) |
| Manage secrets | Contents (write) |
| Manage variables | Contents (write) |
| Manage runners | Administration (write) |
| Cancel runs | Contents (write) |

---

## Reference

- API Version: 2022-11-28
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- Endpoints listed in [api-endpoints.md](api-endpoints.md)

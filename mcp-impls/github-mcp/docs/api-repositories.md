# Repositories API

## Overview

The Repositories API allows you to manage GitHub repositories, including creating, updating, deleting, and querying repository information.

### Base Endpoint
```
https://api.github.com/repos/{owner}/{repo}
```

---

## Repository Information

### Get a repository

#### Endpoint
```
GET /repos/{owner}/{repo}
```

#### Response
```json
{
  "id": 1296269,
  "node_id": "MDEwOlJlcG9zaXRvcnkxMjk2MjY5",
  "name": "Hello-World",
  "full_name": "octocat/Hello-World",
  "private": false,
  "owner": {
    "login": "octocat",
    "id": 1,
    "avatar_url": "https://github.com/images/error/octocat_happy.gif"
  },
  "html_url": "https://github.com/octocat/Hello-World",
  "description": "This your first repo!",
  "fork": false,
  "url": "https://api.github.com/repos/octocat/Hello-World",
  "created_at": "2011-01-26T19:01:12Z",
  "updated_at": "2011-01-26T19:14:43Z",
  "pushed_at": "2011-01-26T19:06:43Z",
  "git_url": "git:github.com/octocat/Hello-World.git",
  "ssh_url": "git@github.com:octocat/Hello-World.git",
  "clone_url": "https://github.com/octocat/Hello-World.git",
  "svn_url": "https://svn.github.com/octocat/Hello-World",
  "homepage": "https://github.com",
  "language": null,
  "forks_count": 9,
  "stargazers_count": 80,
  "watchers_count": 80,
  "size": 108,
  "default_branch": "main",
  "open_issues_count": 0,
  "is_template": false,
  "topics": ["octocat", "atom", "electron"],
  "has_issues": true,
  "has_wiki": true,
  "has_pages": false,
  "has_discussions": false,
  "mirror_url": null,
  "archived": false,
  "disabled": false,
  "visibility": "public",
  "license": {
    "key": "mit",
    "name": "MIT License",
    "spdx_id": "MIT",
    "url": "https://api.github.com/licenses/mit"
  },
  "allow_forking": true,
  "web_commit_signoff_required": false
}
```

---

## List repositories

### List repositories for the authenticated user

#### Endpoint
```
GET /user/repos
GET /users/{username}/repos
GET /orgs/{org}/repos
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `visibility` | string | No | Filter by visibility: `all`, `public`, `private` |
| `affiliation` | string | No | Filter by affiliation: `owner`, `collaborator`, `organization_member` |
| `type` | string | No | Filter by type: `all`, `owner`, `member` |
| `sort` | string | No | Sort by: `created`, `updated`, `full_name` |
| `direction` | string | No | Direction: `asc` or `desc` |
| `per_page` | integer | No | Results per page (max 100, default 30) |
| `page` | integer | No | Page number (default 1) |

---

## Create a repository

### For a user

#### Endpoint
```
POST /user/repos
```

### For an organization

#### Endpoint
```
POST /orgs/{org}/repos
```

#### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | The name of the repository |
| `description` | string | No | A short description |
| `homepage` | string | No | A URL with more information |
| `private` | boolean | No | `true` for private (default: `false`) |
| `visibility` | string | No | Can be `public`, `private`, or `internal` |
| `has_issues` | boolean | No | Enable issues (default: `true`) |
| `has_projects` | boolean | No | Enable projects (default: `true`) |
| `has_wiki` | boolean | No | Enable wiki (default: `true`) |
| `is_template` | boolean | No | Make this a template repository |
| `gitignore_template` | string | No | Gitignore template name |
| `license_template` | string | No | License template name |
| `auto_init` | boolean | No | Initialize with README (default: `false`) |

#### Request Example
```bash
curl -L \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/user/repos \
  -d '{
    "name": "my-repo",
    "description": "My new repository",
    "private": false,
    "auto_init": true
  }'
```

---

## Update a repository

### Endpoint
```
PATCH /repos/{owner}/{repo}
```

### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | No | The name of the repository |
| `description` | string | No | A short description |
| `homepage` | string | No | A URL with more information |
| `private` | boolean | No | `true` to make private |
| `visibility` | string | No | Can be `public`, `private`, or `internal` |
| `has_issues` | boolean | No | Enable/disable issues |
| `has_projects` | boolean | No | Enable/disable projects |
| `has_wiki` | boolean | No | Enable/disable wiki |
| `default_branch` | string | No | Update the default branch |
| `allow_squash_merge` | boolean | No | Allow squash merge |
| `allow_merge_commit` | boolean | No | Allow merge commit |
| `allow_rebase_merge` | boolean | No | Allow rebase merge |
| `delete_branch_on_merge` | boolean | No | Delete branch after merge |
| `squash_merge_commit_title` | string | No | Title for squash merge commits |
| `squash_merge_commit_message` | string | No | Body for squash merge commits |
| `merge_commit_title` | string | No | Title for merge commits |
| `merge_commit_message` | string | No | Body for merge commits |
| `security_and_analysis` | object | No | Security and analysis settings |

---

## Delete a repository

### Endpoint
```
DELETE /repos/{owner}/{repo}
```

**Note:** Deleting a repository requires admin access.

---

## Repository Contents

See [api-contents.md](api-contents.md) for:
- Get repository content
- Create/update file contents
- Delete files
- Get README
- Download archive (tarball/zipball)

---

## Repository Contributors

### List contributors

#### Endpoint
```
GET /repos/{owner}/{repo}/contributors
```

---

## Repository Languages

### List languages

#### Endpoint
```
GET /repos/{owner}/{repo}/languages
```

---

## Repository Teams

### List teams

#### Endpoint
```
GET /repos/{owner}/{repo}/teams
```

---

## Repository Forks

### List forks

#### Endpoint
```
GET /repos/{owner}/{repo}/forks
```

### Create a fork

#### Endpoint
```
POST /repos/{owner}/{repo}/forks
```

---

## Repository Webhooks

See [api-webhooks.md](api-webhooks.md)

---

## Repository Stats

### Get contributors list with additions/deletions

#### Endpoint
```
GET /repos/{owner}/{repo}/stats/contributors
```

### Get commit activity

#### Endpoint
```
GET /repos/{owner}/{repo}/stats/commit_activity
```

### Get code frequency

#### Endpoint
```
GET /repos/{owner}/{repo}/stats/code_frequency
```

### Get participation stats

#### Endpoint
```
GET /repos/{owner}/{repo}/stats/participation
```

### Get punch card

#### Endpoint
```
GET /repos/{owner}/{repo}/stats/punch_card
```

---

## Repository Traffic

### Get clones

#### Endpoint
```
GET /repos/{owner}/{repo}/traffic/clones
```

### Get referrers

#### Endpoint
```
GET /repos/{owner}/{repo}/traffic/popular/referrers
```

### Get paths

#### Endpoint
```
GET /repos/{owner}/{repo}/traffic/popular/paths
```

### Get views

#### Endpoint
```
GET /repos/{owner}/{repo}/traffic/views
```

---

## Repository Activity

### Get repository activity

#### Endpoint
```
GET /repos/{owner}/{repo}/activity
```

---

## Repository Tags

### List tags

#### Endpoint
```
GET /repos/{owner}/{repo}/tags
```

---

## Repository Branches

See [api-branches.md](api-endpoints.md) for branch operations.

---

## Repository Transfer

### Transfer a repository

#### Endpoint
```
POST /repos/{owner}/{repo}/transfer
```

#### Request Body
```json
{
  "new_owner": "new-owner",
  "new_name": "new-name"
}
```

---

## Repository Subscription

### Get a repository subscription

#### Endpoint
```
GET /repos/{owner}/{repo}/subscription
```

### Set a repository subscription

#### Endpoint
```
PUT /repos/{owner}/{repo}/subscription
```

#### Request Body
```json
{
  "subscribed": true,
  "ignored": false
}
```

### Delete a repository subscription

#### Endpoint
```
DELETE /repos/{owner}/{repo}/subscription
```

---

## Repository Autolinks

### List autolinks

#### Endpoint
```
GET /repos/{owner}/{repo}/autolinks
```

### Create an autolink

#### Endpoint
```
POST /repos/{owner}/{repo}/autolinks
```

#### Request Body
```json
{
  "key_prefix": "TICKET-",
  "url_template": "https://example.com/TICKET?query=<num>"
}
```

### Get an autolink

#### Endpoint
```
GET /repos/{owner}/{repo}/autolinks/{autolink_id}
```

### Delete an autolink

#### Endpoint
```
DELETE /repos/{owner}/{repo}/autolinks/{autolink_id}
```

---

## Immutable Releases

### Get immutable releases settings

#### Endpoint
```
GET /repos/{owner}/{repo}/immutable-releases
```

### Set immutable releases settings

#### Endpoint
```
PUT /repos/{owner}/{repo}/immutable-releases
```

---

## Vulnerability Alerts

### Get vulnerability alerts

#### Endpoint
```
GET /repos/{owner}/{repo}/vulnerability-alerts
```

### Enable/disable vulnerability alerts

#### Endpoint
```
PUT /repos/{owner}/{repo}/vulnerability-alerts
```

---

## Permissions

| Action | Required Permission |
|--------|---------------------|
| Read repository | Contents (read) or Metadata (read) |
| Create repository | No permissions (for user) or Administration (write) |
| Update repository | Administration or Contents (write) |
| Delete repository | Administration (write) |
| Manage webhooks | Administration (write) |
| Manage topics | Contents (write) |
| Manage automolinks | Administration (write) |

---

## Reference

- [Official Documentation](https://docs.github.com/en/rest/repos/repos)
- API Version: 2022-11-28
- Endpoints listed in [api-endpoints.md](api-endpoints.md)

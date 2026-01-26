# Organizations API

## Overview

The Organizations API allows you to interact with GitHub organizations, including managing organization settings, members, teams, and repositories.

### Base Endpoint
```
https://api.github.com/orgs/{org}
```

---

## List organizations

Lists all organizations, in the order that they were created.

### Endpoint
```
GET /organizations
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `since` | integer | No | An organization ID. Only return organizations with an ID greater than this ID |
| `per_page` | integer | No | Results per page (max 100, default 30) |

**Note:** Pagination is powered exclusively by the `since` parameter. Use the Link header to get the URL for the next page.

### Response
```json
[
  {
    "login": "github",
    "id": 1,
    "node_id": "MDEyOk9yZ2FuaXphdGlvbjE=",
    "url": "https://api.github.com/orgs/github",
    "repos_url": "https://api.github.com/orgs/github/repos",
    "events_url": "https://api.github.com/orgs/github/events",
    "hooks_url": "https://api.github.com/orgs/github/hooks",
    "issues_url": "https://api.github.com/orgs/github/issues",
    "members_url": "https://api.github.com/orgs/github/members{/member}",
    "public_members_url": "https://api.github.com/orgs/github/public_members{/member}",
    "avatar_url": "https://github.com/images/error/octocat_happy.gif",
    "description": "A great organization"
  }
]
```

---

## Get an organization

Gets information about an organization.

### Endpoint
```
GET /orgs/{org}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `org` | string | Yes | The organization name (not case sensitive) |

### Response
```json
{
  "login": "github",
  "id": 1,
  "node_id": "MDEyOk9yZ2FuaXphdGlvbjE=",
  "url": "https://api.github.com/orgs/github",
  "repos_url": "https://api.github.com/orgs/github/repos",
  "events_url": "https://api.github.com/orgs/github/events",
  "hooks_url": "https://api.github.com/orgs/github/hooks",
  "issues_url": "https://api.github.com/orgs/github/issues",
  "members_url": "https://api.github.com/orgs/github/members{/member}",
  "public_members_url": "https://api.github.com/orgs/github/public_members{/member}",
  "avatar_url": "https://github.com/images/error/octocat_happy.gif",
  "description": "A great organization",
  "name": "github",
  "company": "GitHub",
  "blog": "https://github.com/blog",
  "location": "San Francisco",
  "email": "octocat@github.com",
  "twitter_username": "github",
  "is_verified": true,
  "has_organization_projects": true,
  "has_repository_projects": true,
  "public_repos": 2,
  "public_gists": 1,
  "followers": 20,
  "following": 0,
  "html_url": "https://github.com/octocat",
  "created_at": "2008-01-14T04:33:35Z",
  "type": "Organization",
  "total_private_repos": 100,
  "owned_private_repos": 100,
  "private_gists": 81,
  "disk_usage": 10000,
  "collaborators": 8,
  "billing_email": "mona@github.com",
  "plan": {
    "name": "Medium",
    "space": 400,
    "private_repos": 20,
    "filled_seats": 4,
    "seats": 5
  },
  "default_repository_permission": "read",
  "default_repository_branch": "main",
  "members_can_create_repositories": true,
  "two_factor_requirement_enabled": true,
  "members_allowed_repository_creation_type": "all",
  "members_can_create_public_repositories": false,
  "members_can_create_private_repositories": false,
  "members_can_create_internal_repositories": false,
  "members_can_create_pages": true,
  "members_can_create_public_pages": true,
  "members_can_create_private_pages": true,
  "members_can_delete_repositories": true,
  "members_can_change_repo_visibility": true,
  "members_can_invite_outside_collaborators": true,
  "members_can_delete_issues": false,
  "display_commenter_full_name_setting_enabled": false,
  "readers_can_create_discussions": true,
  "members_can_create_teams": true,
  "members_can_view_dependency_insights": true,
  "members_can_fork_private_repositories": false,
  "web_commit_signoff_required": false,
  "updated_at": "2014-03-03T18:58:10Z"
}
```

---

## Update an organization

Updates the organization's profile and member privileges.

### Endpoint
```
PATCH /orgs/{org}
```

### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `billing_email` | string | No | Billing email address (not publicized) |
| `company` | string | No | The company name |
| `email` | string | No | The publicly visible email address |
| `twitter_username` | string | No | The Twitter username of the company |
| `location` | string | No | The location |
| `name` | string | No | The shorthand name of the company |
| `description` | string | No | The description (max 160 characters) |
| `has_organization_projects` | boolean | No | Whether organization projects are enabled |
| `has_repository_projects` | boolean | No | Whether repository projects are enabled |
| `default_repository_permission` | string | No | Default permission level: `read`, `write`, `admin`, `none` |
| `members_can_create_repositories` | boolean | No | Whether members can create repositories |
| `members_can_create_public_repositories` | boolean | No | Whether members can create public repositories |
| `members_can_create_private_repositories` | boolean | No | Whether members can create private repositories |
| `members_can_create_internal_repositories` | boolean | No | Whether members can create internal repositories |
| `members_can_create_pages` | boolean | No | Whether members can create GitHub Pages sites |
| `members_can_create_public_pages` | boolean | No | Whether members can create public Pages sites |
| `members_can_create_private_pages` | boolean | No | Whether members can create private Pages sites |
| `members_can_fork_private_repositories` | boolean | No | Whether members can fork private repositories |
| `web_commit_signoff_required` | boolean | No | Whether contributors must sign off on web commits |

### Required Permissions
- **Administration** organization permissions (write)

---

## Delete an organization

Deletes an organization and all its repositories.

### Endpoint
```
DELETE /orgs/{org}
```

**Note:** The organization login will be unavailable for 90 days after deletion.

### Required Permissions
- **Administration** organization permissions (write)

---

## Organization Members API

### List members
```
GET /orgs/{org}/members
```

### Check if a user is a member
```
GET /orgs/{org}/members/{username}
```

### Remove a member
```
DELETE /orgs/{org}/members/{username}
```

### Get membership for a user
```
GET /orgs/{org}/memberships/{username}
```

### Add/update membership
```
PUT /orgs/{org}/memberships/{username}
```
```json
{
  "role": "admin"
}
```

### Remove membership
```
DELETE /orgs/{org}/memberships/{username}
```

### List public members
```
GET /orgs/{org}/public_members
```

### Check public membership
```
GET /orgs/{org}/public_members/{username}
```

### Set public membership
```
PUT /orgs/{org}/public_members/{username}
```

### Remove public membership
```
DELETE /orgs/{org}/public_members/{username}
```

---

## Organization Outside Collaborators API

### List outside collaborators
```
GET /orgs/{org}/outside_collaborators
```

### Convert member to outside collaborator
```
PUT /orgs/{org}/outside_collaborators/{username}
```

### Remove outside collaborator
```
DELETE /orgs/{org}/outside_collaborators/{username}
```

---

## Organization Webhooks API

### List webhooks
```
GET /orgs/{org}/hooks
```

### Create a webhook
```
POST /orgs/{org}/hooks
```

### Get a webhook
```
GET /orgs/{org}/hooks/{hook_id}
```

### Update a webhook
```
PATCH /orgs/{org}/hooks/{hook_id}
```

### Delete a webhook
```
DELETE /orgs/{org}/hooks/{hook_id}
```

### Ping a webhook
```
POST /orgs/{org}/hooks/{hook_id}/pings
```

---

## List organizations for authenticated user

### Endpoint
```
GET /user/orgs
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `per_page` | integer | No | Results per page (max 100, default 30) |
| `page` | integer | No | Page number (default 1) |

---

## List organizations for a user

List public organization memberships for the specified user.

### Endpoint
```
GET /users/{username}/orgs
```

**Note:** This only lists public memberships. Use `GET /user/orgs` for the authenticated user to get all memberships.

---

## Related Endpoints

| Endpoint | Description |
|----------|-------------|
| `/orgs/{org}/installations` | List app installations |
| `/orgs/{org}/repos` | List repositories |
| `/orgs/{org}/teams` | List teams |
| `/orgs/{org}/invitations` | List pending invitations |
| `/orgs/{org}/settings/immutable-releases` | Get immutable releases settings |
| `/orgs/{org}/blocks/{username}` | Block/unblock users |
| `/orgs/{org}/hooks` | Manage webhooks |

---

## Permissions

| Action | Required Permission |
|--------|---------------------|
| Read organization | No permissions required (public info) |
| Update organization | Administration (write) |
| Delete organization | Administration (write) |
| Manage members | Members (read/write) |

---

## Reference

- [Official Documentation](https://docs.github.com/en/rest/orgs/orgs)
- API Version: 2022-11-28

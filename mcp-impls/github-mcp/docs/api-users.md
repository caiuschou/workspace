# Users API

## Overview

The Users API allows you to get public and private information about authenticated users.

### Base Endpoint
```
https://api.github.com/users
```

---

## Get the authenticated user

### Endpoint
```
GET /user
```

### Response with public and private profile information

**Note:** OAuth app tokens and personal access tokens (classic) need the `user` scope for private profile information.

```json
{
  "login": "octocat",
  "id": 1,
  "node_id": "MDQ6VXNlcjE=",
  "avatar_url": "https://github.com/images/error/octocat_happy.gif",
  "gravatar_id": "",
  "url": "https://api.github.com/users/octocat",
  "html_url": "https://github.com/octocat",
  "followers_url": "https://api.github.com/users/octocat/followers",
  "following_url": "https://api.github.com/users/octocat/following{/other_user}",
  "gists_url": "https://api.github.com/users/octocat/gists{/gist_id}",
  "starred_url": "https://api.github.com/users/octocat/starred{/owner}{/repo}",
  "subscriptions_url": "https://api.github.com/users/octocat/subscriptions",
  "organizations_url": "https://api.github.com/users/octocat/orgs",
  "repos_url": "https://api.github.com/users/octocat/repos",
  "events_url": "https://api.github.com/users/octocat/events{/privacy}",
  "received_events_url": "https://api.github.com/users/octocat/received_events",
  "type": "User",
  "site_admin": false,
  "name": "monalisa octocat",
  "company": "GitHub",
  "blog": "https://github.com/blog",
  "location": "San Francisco",
  "email": "octocat@github.com",
  "hireable": false,
  "bio": "There once was...",
  "twitter_username": "monatheoctocat",
  "public_repos": 2,
  "public_gists": 1,
  "followers": 20,
  "following": 0,
  "created_at": "2008-01-14T04:33:35Z",
  "updated_at": "2008-01-14T04:33:35Z",
  "private_gists": 81,
  "total_private_repos": 100,
  "owned_private_repos": 100,
  "disk_usage": 10000,
  "collaborators": 8,
  "two_factor_authentication": true,
  "plan": {
    "name": "Medium",
    "space": 400,
    "private_repos": 20,
    "collaborators": 0
  }
}
```

---

## Update the authenticated user

### Endpoint
```
PATCH /user
```

### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | No | The new name of the user |
| `email` | string | No | The publicly visible email address |
| `blog` | string | No | The new blog URL |
| `twitter_username` | string or null | No | The new Twitter username |
| `company` | string | No | The new company |
| `location` | string | No | The new location |
| `hireable` | boolean | No | The new hiring availability |
| `bio` | string | No | The new short biography |

### Request Example
```bash
curl -L \
  -X PATCH \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/user \
  -d '{"blog":"https://github.com/blog","name":"monalisa octocat"}'
```

### Required Permissions
- **Profile** user permissions (write)

---

## Get a user using their ID

Provides publicly available information about someone with a GitHub account using their durable user ID.

### Endpoint
```
GET /user/{account_id}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `account_id` | integer | Yes | The user's account ID |

**Note:** The user ID is durable and doesn't change, unlike the login which can change.

---

## List users

Lists all users, in the order that they signed up on GitHub. This list includes personal user accounts and organization accounts.

### Endpoint
```
GET /users
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `since` | integer | No | A user ID. Only return users with an ID greater than this ID |
| `per_page` | integer | No | Results per page (max 100, default 30) |

**Note:** Pagination is powered exclusively by the `since` parameter.

### Response
```json
[
  {
    "login": "octocat",
    "id": 1,
    "node_id": "MDQ6VXNlcjE=",
    "avatar_url": "https://github.com/images/error/octocat_happy.gif",
    "gravatar_id": "",
    "url": "https://api.github.com/users/octocat",
    "html_url": "https://github.com/octocat",
    "followers_url": "https://api.github.com/users/octocat/followers",
    "following_url": "https://api.github.com/users/octocat/following{/other_user}",
    "gists_url": "https://api.github.com/users/octocat/gists{/gist_id}",
    "starred_url": "https://api.github.com/users/octocat/starred{/owner}{/repo}",
    "subscriptions_url": "https://api.github.com/users/octocat/subscriptions",
    "organizations_url": "https://api.github.com/users/octocat/orgs",
    "repos_url": "https://api.github.com/users/octocat/repos",
    "events_url": "https://api.github.com/users/octocat/events{/privacy}",
    "received_events_url": "https://api.github.com/users/octocat/received_events",
    "type": "User",
    "site_admin": false
  }
]
```

---

## Get a user

Provides publicly available information about someone with a GitHub account.

### Endpoint
```
GET /users/{username}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `username` | string | Yes | The handle for the GitHub user account |

### Response
```json
{
  "login": "octocat",
  "id": 1,
  "node_id": "MDQ6VXNlcjE=",
  "avatar_url": "https://github.com/images/error/octocat_happy.gif",
  "gravatar_id": "",
  "url": "https://api.github.com/users/octocat",
  "html_url": "https://github.com/octocat",
  "followers_url": "https://api.github.com/users/octocat/followers",
  "following_url": "https://api.github.com/users/octocat/following{/other_user}",
  "gists_url": "https://api.github.com/users/octocat/gists{/gist_id}",
  "starred_url": "https://api.github.com/users/octocat/starred{/owner}{/repo}",
  "subscriptions_url": "https://api.github.com/users/octocat/subscriptions",
  "organizations_url": "https://api.github.com/users/octocat/orgs",
  "repos_url": "https://api.github.com/users/octocat/repos",
  "events_url": "https://api.github.com/users/octocat/events{/privacy}",
  "received_events_url": "https://api.github.com/users/octocat/received_events",
  "type": "User",
  "site_admin": false,
  "name": "monalisa octocat",
  "company": "GitHub",
  "blog": "https://github.com/blog",
  "location": "San Francisco",
  "email": "octocat@github.com",
  "hireable": false,
  "bio": "There once was...",
  "twitter_username": "monatheoctocat",
  "public_repos": 2,
  "public_gists": 1,
  "followers": 20,
  "following": 0,
  "created_at": "2008-01-14T04:33:35Z",
  "updated_at": "2008-01-14T04:33:35Z"
}
```

---

## Get contextual information for a user

Provides hovercard information with contextual data about a user in relation to their pull requests, issues, repositories, and organizations.

### Endpoint
```
GET /users/{username}/hovercard
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `username` | string | Yes | The handle for the GitHub user account |
| `subject_type` | string | No | Can be `organization`, `repository`, `issue`, `pull_request` |
| `subject_id` | string | No | The ID for the `subject_type` you specified |

### Response
```json
{
  "contexts": [
    {
      "message": "Owns this repository",
      "octicon": "repo"
    }
  ]
}
```

**Note:** This endpoint does not work with GitHub App user access tokens, GitHub App installation access tokens, or fine-grained personal access tokens.

---

## Emails API

### List emails for the authenticated user
```
GET /user/emails
```

### Add email address(es)
```
POST /user/emails
```
```json
{
  "emails": ["octocat@github.com", "octocat@example.com"],
  "visibility": "private"
}
```

### Delete email address(es)
```
DELETE /user/emails
```
```json
{
  "emails": ["octocat@github.com"]
}
```

### List public email addresses
```
GET /user/public_emails
```

---

## Followers API

### List followers of a user
```
GET /users/{username}/followers
```

### List people a user follows
```
GET /users/{username}/following
```

### Check if a user follows another user
```
GET /user/following/{target_user}
GET /users/{username}/following/{target_user}
```

### Follow a user
```
PUT /user/following/{username}
```

### Unfollow a user
```
DELETE /user/following/{username}
```

---

## SSH Keys & GPG Keys API

### List SSH keys for the authenticated user
```
GET /user/keys
```

### Create a SSH key
```
POST /user/keys
```
```json
{
  "title": "octocat@macbook-pro",
  "key": "ssh-rsa AAA...",
  "read_only": true
}
```

### Get a SSH key
```
GET /user/keys/{key_id}
```

### Delete a SSH key
```
DELETE /user/keys/{key_id}
```

### List GPG keys for the authenticated user
```
GET /user/gpg_keys
```

### Add a GPG key
```
POST /user/gpg_keys
```
```json
{
  "armored_public_key": "-----BEGIN PGP PUBLIC KEY BLOCK-----..."
}
```

### Get a GPG key
```
GET /user/gpg_keys/{gpg_key_id}
```

### Delete a GPG key
```
DELETE /user/gpg_keys/{gpg_key_id}
```

### List SSH signing keys
```
GET /user/ssh_signing_keys
POST /user/ssh_signing_keys
GET /user/ssh_signing_keys/{ssh_signing_key_id}
DELETE /user/ssh_signing_keys/{ssh_signing_key_id}
```

---

## Blocking Users API

### List users blocked by the authenticated user
```
GET /user/blocks
```

### Check if a user is blocked
```
GET /user/blocks/{username}
```

### Block a user
```
PUT /user/blocks/{username}
```

### Unblock a user
```
DELETE /user/blocks/{username}
```

---

## Social Accounts API

### List social accounts for the authenticated user
```
GET /user/social_accounts
```

### Add social accounts
```
POST /user/social_accounts
```
```json
{
  "account_urls": ["https://twitter.com/account"]
}
```

### Delete social accounts
```
DELETE /user/social_accounts
```
```json
{
  "account_urls": ["https://twitter.com/account"]
}
```

---

## Related Endpoints

| Endpoint | Description |
|----------|-------------|
| `/user/repos` | List repositories for the authenticated user |
| `/user/orgs` | List organizations for the authenticated user |
| `/user/installations` | List GitHub App installations |
| `/users/{username}/repos` | List repositories for a user |
| `/users/{username}/orgs` | List organizations for a user |
| `/users/{username}/keys` | List public SSH keys for a user |

---

## Permissions

| Action | Required Permission |
|--------|---------------------|
| Get authenticated user | None |
| Update authenticated user | Profile (write) |
| Get public user info | None |
| Manage emails | Email (read/write) |
| Manage keys | Account administration (write) |
| Block users | None |

---

## Reference

- [Official Documentation](https://docs.github.com/en/rest/users/users)
- API Version: 2022-11-28

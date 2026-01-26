# GitHub REST API Authentication

## Overview

GitHub REST API supports multiple authentication methods depending on your use case.

### Base URL
```
https://api.github.com
```

### API Version Header (Recommended)
```
X-GitHub-Api-Version: 2022-11-28
```

---

## Authentication Methods

### 1. Personal Access Tokens (Recommended)

#### Fine-grained Personal Access Tokens

Fine-grained PATs provide more granular control and enhanced security.

**Creating a fine-grained PAT:**
1. Go to Settings → Developer settings → Personal access tokens → Fine-grained tokens
2. Click "Generate new token (classic)"
3. Configure:
   - Token name
   - Expiration (recommended: 90 days or less)
   - Resource owner (account or organization)
   - Repository access permissions
   - Permissions per resource (Contents, Issues, Pull Requests, etc.)

**Usage:**
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.github.com/user
```

#### Classic Personal Access Tokens

Classic PATs have broader scopes (`repo`, `user`, `admin:org`, etc.).

**Scopes:**
- `repo` - Full control of private repositories
- `repo:status` - Access commit status
- `repo_deployment` - Access deployment status
- `public_repo` - Access public repositories
- `admin:org` - Manage org
- `user` - Read/write user profile
- `user:email` - Read user email
- `user:follow` - Follow/unfollow users
- `gist` - Create gists
- `notifications` - Access notifications
- `read:org` - Read org and team membership
- `repo:invite` - Accept repo invites
- `security_events` - Read/write security events

**Usage:**
```bash
curl -H "Authorization: token YOUR_CLASSIC_TOKEN" \
  https://api.github.com/user
```

---

### 2. OAuth2 Tokens

OAuth2 allows applications to act on behalf of users.

**Authorization URL:**
```
https://github.com/login/oauth/authorize
```

**Access Token URL:**
```
https://github.com/login/oauth/access_token
```

**Parameters:**
| Parameter | Description |
|-----------|-------------|
| `client_id` | Your OAuth app client ID |
| `redirect_uri` | URL to redirect after authorization |
| `scope` | Scopes requested |
| `state` | Random string to prevent CSRF |
| `allow_signup` | Allow signup during flow (default: `true`) |

**Usage:**
```bash
curl -H "Authorization: Bearer YOUR_OAUTH_TOKEN" \
  https://api.github.com/user
```

---

### 3. GitHub Apps

GitHub Apps are the recommended way to authenticate as an application.

#### JWT (JSON Web Token)

For server-to-server requests (acting as the app installation):

```javascript
const jwt = require('jsonwebtoken');

const payload = {
  iss: APP_ID,           // GitHub App ID
  iat: Math.floor(Date.now() / 1000),
  exp: Math.floor(Date.now() / 1000) + 60  // Expire in 1 minute
};

const token = jwt.sign(payload, PRIVATE_KEY, { algorithm: 'RS256' });
```

```bash
curl -H "Authorization: Bearer YOUR_JWT" \
  -H "Accept: application/vnd.github+json" \
  https://api.github.com/app/installations
```

#### Installation Access Token

For acting on a specific repository installation:

1. Get installations with JWT
2. Get installation access token
3. Use access token for API requests

```bash
# Get installation access token
curl -X POST \
  -H "Authorization: Bearer YOUR_JWT" \
  -H "Accept: application/vnd.github+json" \
  https://api.github.com/app/installations/INSTALLATION_ID/access_tokens
```

**Response:**
```json
{
  "token": "ghs_...",
  "expires_at": "2024-01-21T00:00:00Z",
  "permissions": {
    "contents": "write",
    "issues": "read"
  },
  "repository_selection": "all"
}
```

```bash
# Use installation access token
curl -H "Authorization: Bearer INSTALLATION_TOKEN" \
  https://api.github.com/repos/owner/repo/issues
```

---

### 4. Basic Authentication (Deprecated)

Basic authentication using username and password is **no longer supported** for API requests.

Use PATs or OAuth tokens instead.

---

## Authentication Header Formats

| Format | Example | Use Case |
|--------|---------|----------|
| `Authorization: Bearer <token>` | `Bearer ghp_xxx` | Fine-grained PATs, OAuth, Installation tokens |
| `Authorization: token <token>` | `token ghp_xxx` | Classic PATs |
| `Authorization: Basic <base64>` | Deprecated | No longer supported |

**Recommended:** Always use `Authorization: Bearer <token>`

---

## Rate Limits by Authentication

| Authentication Type | Rate Limit |
|---------------------|------------|
| Fine-grained PAT | 5,000 requests/hour |
| Classic PAT | 5,000 requests/hour |
| OAuth App Token | 5,000 requests/hour |
| GitHub App Installation | 5,000 requests/hour (per installation) |
| GitHub App (JWT) | 5,000 requests/hour |
| Unauthenticated | 60 requests/hour |

### Check Rate Limit
```bash
curl https://api.github.com/rate_limit
```

**Response:**
```json
{
  "resources": {
    "core": {
      "limit": 5000,
      "remaining": 4999,
      "reset": 1705843200,
      "used": 1
    }
  }
}
```

---

## Permissions for Fine-grained PATs

### Repository Permissions

| Permission | Read | Write |
|------------|------|-------|
| Administration | ✓ | ✓ |
| Actions | ✓ | ✓ |
| Checks | ✓ | ✓ |
| Code scanning alerts | ✓ | ✓ |
| Codespaces | ✓ | ✓ |
| Commit statuses | ✓ | ✓ |
| Contents | ✓ | ✓ |
| Dependabot alerts | ✓ | ✓ |
| Deployments | ✓ | ✓ |
| Environments | ✓ | ✓ |
| Issues | ✓ | ✓ |
| Merge queues | ✓ | ✓ |
| Metadata | ✓ | - |
| Pages | ✓ | ✓ |
| Pull requests | ✓ | ✓ |
| Repository projects | ✓ | ✓ |
| Secret scanning alerts | ✓ | ✓ |
| Secrets | - | ✓ |
| Security events | ✓ | ✓ |
| Workflows | - | ✓ |

### Organization Permissions

| Permission | Read | Write |
|------------|------|-------|
| Administration | ✓ | ✓ |
| Codespaces packages | ✓ | ✓ |
| Custom repository roles | ✓ | ✓ |
| Members | ✓ | ✓ |
| Organization packages | ✓ | ✓ |
| Projects | ✓ | ✓ |
| Team discussions | ✓ | ✓ |
| Teams | ✓ | ✓ |
| Webhooks | ✓ | ✓ |

---

## Troubleshooting

### Common Errors

| Status | Error | Solution |
|--------|-------|----------|
| 401 | Bad credentials | Check token is valid |
| 403 | Resource not accessible | Check token has required permissions |
| 403 | Rate limit exceeded | Wait or use authenticated requests |
| 404 | Not found | Check resource exists and token has access |

### Testing Authentication
```bash
# Test your token
curl -v \
  -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.github.com/user

# Check token permissions
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.github.com/user/repos
```

---

## Best Practices

1. **Use fine-grained PATs** - Minimize permissions to only what's needed
2. **Set expiration dates** - Don't create tokens that never expire
3. **Store tokens securely** - Use environment variables or secret managers
4. **Use GitHub Apps** - For integrations, GitHub Apps are more secure
5. **Monitor rate limits** - Check remaining quota in API responses
6. **Handle 401 responses** - Token may have expired or been revoked
7. **Use conditional requests** - Use `ETag`/`If-None-Match` to save quota

---

## Reference

- [Authenticating to the REST API](https://docs.github.com/en/rest/authentication/authenticating-to-the-rest-api)
- [Fine-grained PATs](https://docs.github.com/en/rest/authentication/endpoints-available-for-fine-grained-personal-access-tokens)
- [GitHub Apps](https://docs.github.com/en/rest/authentication/authenticating-to-the-rest-api)
- API Version: 2022-11-28

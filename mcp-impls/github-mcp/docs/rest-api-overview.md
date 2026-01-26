# GitHub REST API Documentation

## Overview

GitHub REST API allows you to create integrations, retrieve data, and automate your workflows.

### Base URL
```
https://api.github.com
```

### API Version
Current version: `2022-11-28` (latest)

### Authentication

GitHub REST API supports multiple authentication methods:

1. **Bearer Token (Personal Access Token)**
   ```
   Authorization: Bearer <token>
   ```
   Or via HTTP header:
   ```
   Authorization: token <token>
   ```

2. **OAuth2 Token**
   ```
   Authorization: Bearer <oauth_token>
   ```

3. **GitHub Apps** - JWT and Installation Access Tokens

4. **Basic Authentication** (deprecated for PATs, use Bearer instead)

### Rate Limits

| Authentication Type | Rate Limit |
|---------------------|------------|
| Authenticated (PAT/App) | 5,000 requests/hour |
| Unauthenticated | 60 requests/hour |

Rate limit status can be checked via:
```
GET /rate_limit
```

### Response Headers

- `X-RateLimit-Limit`: Request limit per hour
- `X-RateLimit-Remaining`: Remaining requests
- `X-RateLimit-Reset`: Unix timestamp when limit resets
- `X-GitHub-Request-Id`: Unique request ID

### Pagination

Most list endpoints support pagination via:
- `page`: Page number (default: 1)
- `per_page`: Items per page (max: 100, default: 30)

Link header provides navigation:
```
Link: <url?page=2>; rel="next", <url?page=10>; rel="last"
```

## API Categories

### Core Resources

| Category | Description |
|----------|-------------|
| **Repositories** | Manage repos, contents, branches, webhooks |
| **Issues** | Issues, comments, labels, milestones, assignees |
| **Pull Requests** | PRs, reviews, comments, files, merge |
| **Users** | User profiles, emails, followers, keys |
| **Organizations** | Orgs, members, teams, webhooks |
| **Search** | Code, commits, issues, repos, users, topics |
| **Git** | Blobs, commits, refs, tags, trees |
| **Actions** | Workflows, runs, artifacts, secrets, variables |
| **Checks** | Check runs and check suites |
| **Releases** | Releases and release assets |
| **Projects** | Projects (classic and V2) |
| **Codespaces** | Codespaces management and secrets |
| **Security** | Code scanning, secret scanning, advisories |

### Common Patterns

#### List resources
```http
GET /repos/{owner}/{repo}/issues?page=1&per_page=30&state=open
```

#### Get single resource
```http
GET /repos/{owner}/{repo}/issues/{issue_number}
```

#### Create resource
```http
POST /repos/{owner}/{repo}/issues
Content-Type: application/json

{
  "title": "New issue",
  "body": "Issue description"
}
```

#### Update resource
```http
PATCH /repos/{owner}/{repo}/issues/{issue_number}
Content-Type: application/json

{
  "state": "closed"
}
```

#### Delete resource
```http
DELETE /repos/{owner}/{repo}/issues/{issue_number}
```

### Conditional Requests

Use `ETag` or `Last-Modified` headers:
```http
If-None-Match: "1234567890abcdef"
If-Modified-Since: Wed, 21 Oct 2015 07:28:00 GMT
```

Returns `304 Not Modified` if resource unchanged.

### Timezones

All timestamps are in UTC (ISO 8601 format):
```
2025-01-21T12:34:56Z
```

### Best Practices

1. **Use conditional requests** to save rate limit
2. **Paginate through results** for large lists
3. **Use webhooks** instead of polling for events
4. **Cache responses** when appropriate
5. **Handle rate limits gracefully** with `Retry-After` header
6. **Use appropriate authentication** for higher limits
7. **Batch operations** where possible

### Client Libraries

- **Octokit.js** - JavaScript/TypeScript
- **Octokit.rb** - Ruby
- **PyGithub** - Python
- **go-github** - Go
- **octokit** - Rust ([https://github.com/XAMPPRocky/octocrab](https://github.com/XAMPPRocky/octocrab))

### OpenAPI Specification

GitHub provides an OpenAPI description:
```
https://api.github.com/spec
```

## Documentation Links

- [Getting Started](https://docs.github.com/en/rest/overview/getting-started-with-the-rest-api)
- [Authentication](https://docs.github.com/en/rest/authentication/authenticating-to-the-rest-api)
- [Rate Limits](https://docs.github.com/en/rest/overview/rate-limits-for-the-rest-api)
- [Pagination](https://docs.github.com/en/rest/using-the-rest-api/using-pagination-in-the-rest-api)
- [Best Practices](https://docs.github.com/en/rest/guides/best-practices-for-using-the-rest-api)
- [Webhooks](https://docs.github.com/en/rest/webhooks/about-webhooks)

## References

- Source: [GitHub REST API Documentation](https://docs.github.com/en/rest)
- API Version: 2022-11-28
- Last updated: 2025-01-21

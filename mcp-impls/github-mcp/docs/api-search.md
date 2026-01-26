# Search API

## Overview

The Search API allows you to search for code, commits, issues, pull requests, repositories, users, and topics across GitHub.

### Base Endpoint
```
https://api.github.com/search
```

### Rate Limits
- **Authenticated**: 30 requests per minute
- **Unauthenticated**: 10 requests per minute

Check your rate limit:
```
GET /rate_limit
```

---

## Search code

### Endpoint
```
GET /search/code?q={query}{&page,per_page,sort,order}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | The search query |
| `sort` | string | No | `indexed` (default) |
| `order` | string | No | `desc` (default) or `asc` |
| `per_page` | integer | No | Results per page (max 100, default 30) |
| `page` | integer | No | Page number (default 1) |

### Query Qualifiers

| Qualifier | Example | Description |
|-----------|---------|-------------|
| `in` | `in:file` | Search in `file` or `path` |
| `language` | `language:python` | Filter by language |
| `fork` | `fork:true` | Include forks in search |
| `size` | `size:1000` | Search by file size (e.g., `>1000`, `<100`) |
| `path` | `path:src` | Search in path |
| `extension` | `extension:md` | Search by file extension |
| `user`/`repo` | `repo:owner/name` | Scope to repository |
| `org` | `org:github` | Scope to organization |

### Query Examples
```
# Search in a specific repository
q=addClass repo:jquery/jquery

# Search by language and file size
q=language:python size:>1000

# Search in a specific path
q=API token path:src/

# Search by extension
q=build config extension:yaml
```

### Request Example
```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "https://api.github.com/search/code?q=addClass+repo:jquery/jquery"
```

### Response
```json
{
  "total_count": 47,
  "incomplete_results": false,
  "items": [
    {
      "name": "action.js",
      "path": "src/action.js",
      "sha": "d28f86aefb9fb5e046eb2f3e43963a93bc1b0a43",
      "url": "https://api.github.com/repos/jquery/jquery/contents/src/action.js",
      "git_url": "https://api.github.com/repos/jquery/jquery/git/blobs/d28f86aefb9fb5e046eb2f3e43963a93bc1b0a43",
      "html_url": "https://github.com/jquery/jquery/blob/master/src/action.js",
      "repository": {
        "id": 1296269,
        "name": "jquery",
        "full_name": "jquery/jquery",
        "private": false
      }
    }
  ]
}
```

---

## Search commits

### Endpoint
```
GET /search/commits?q={query}{&page,per_page,sort,order}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | The search query |
| `sort` | string | No | `author-date` or `committer-date` |
| `order` | string | No | `desc` (default) or `asc` |
| `per_page` | integer | No | Results per page (max 100, default 30) |
| `page` | integer | No | Page number (default 1) |

### Query Qualifiers

| Qualifier | Example | Description |
|-----------|---------|-------------|
| `hash` | `hash:4029a3` | Search by SHA |
| `author` | `author:octocat` | Filter by author |
| `committer` | `committer:octocat` | Filter by committer |
| `author-name`/`committer-name` | `author-name:Octo` | Filter by name |
| `author-email`/`committer-email` | `author-email:users@noreply.github.com` | Filter by email |
| `author-date`/`committer-date` | `author-date:>2024-01-01` | Filter by date |
| `merge` | `merge:true` | Filter merge commits |
| `is` | `is:public` | Filter by `public`/`private` |
| `user`/`repo`/`org` | `repo:owner/name` | Scope to repo/user/org |

### Query Examples
```
# Search by author
q=author:octocat

# Search by date range
q=author-date:>2024-01-01 author-date:<2024-12-31

# Search merge commits
q=merge:true repo:owner/name

# Search by hash
q=hash:4029a3
```

---

## Search issues and pull requests

### Endpoint
```
GET /search/issues?q={query}{&page,per_page,sort,order}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | The search query (requires at least one qualifier) |
| `sort` | string | No | `comments`, `created`, `updated` (default: `best match`) |
| `order` | string | No | `desc` (default) or `asc` |
| `per_page` | integer | No | Results per page (max 100, default 30) |
| `page` | integer | No | Page number (default 1) |

### Query Qualifiers

| Qualifier | Example | Description |
|-----------|---------|-------------|
| `in` | `in:title` | Search in `title`, `body`, or `comments` |
| `type` | `type:issue` or `type:pr` | Filter by type |
| `is` | `is:open` or `is:closed` | Filter by state |
| `author` | `author:octocat` | Filter by creator |
| `assignee` | `assignee:octocat` | Filter by assignee |
| `mentions` | `mentions:octocat` | Filter by @mentioned user |
| `commenter` | `commenter:octocat` | Filter by commenter |
| `involves` | `involves:octocat` | Filter by author, assignee, or mention |
| `state` | `state:open` | Filter by `open` or `closed` |
| `labels` | `labels:bug` | Filter by label |
| `no` | `no:label` | Exclude results |
| `language` | `language:python` | Filter by repo language |
| `created` | `created:>2024-01-01` | Filter by creation date |
| `updated` | `updated:<2024-12-31` | Filter by update date |
| `comments` | `comments:>10` | Filter by comment count |
| `user`/`repo`/`org` | `repo:owner/name` | Scope to repo/user/org |

### Query Examples
```
# Search open issues in a repo
q=repo:owner/name type:issue state:open

# Search PRs by author
q=author:octocat type:pr state:open

# Search by label and comments
q=labels:bug comments:>5

# Search by date
q=created:>2024-01-01 type:pr

# Search in title only
q=in:title error repo:owner/name

# Search multiple repos
q=repo:owner/repo1+repo:owner/repo2 type:issue
```

---

## Search repositories

### Endpoint
```
GET /search/repositories?q={query}{&page,per_page,sort,order}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | The search query |
| `sort` | string | No | `stars`, `forks`, `help-wanted-issues`, `updated` |
| `order` | string | No | `desc` (default) or `asc` |
| `per_page` | integer | No | Results per page (max 100, default 30) |
| `page` | integer | No | Page number (default 1) |

### Query Qualifiers

| Qualifier | Example | Description |
|-----------|---------|-------------|
| `in` | `in:name` | Search in `name`, `description`, or `readme` |
| `size` | `size:>1000` | Filter by repo size (KB) |
| `forks` | `forks:>100` | Filter by fork count |
| `fork` | `fork:true`/`only` | Include/exclude forks |
| `stars` | `stars:>1000` | Filter by star count |
| `language` | `language:python` | Filter by primary language |
| `archived` | `archived:true` | Filter archived repos |
| `good-first-issues` | `good-first-issues:>5` | Filter by good first issues count |
| `help-wanted-issues` | `help-wanted-issues:>5` | Filter by help wanted issues count |
| `topics` | `topics:>5` | Filter by topic count |
| `is` | `is:public` | Filter by `public`, `private`, `fork` |
| `pushed` | `pushed:>2024-01-01` | Filter by last push date |
| `created` | `created:>2024-01-01` | Filter by creation date |
| `user`/`org` | `user:octocat` | Filter by owner |
| `topic` | `topic:api` | Filter by topic |

### Query Examples
```
# Search by name
q=github api in:name

# Search by language and stars
q=language:python stars:>1000

# Search repos with good first issues
q=good-first-issues:>5

# Search by org
q=org:github language:javascript

# Search by topic
q=topic:machine-learning stars:>100

# Find forks of a repo
q=repo:torvalds/linux fork:true
```

---

## Search users

### Endpoint
```
GET /search/users?q={query}{&page,per_page,sort,order}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | The search query |
| `sort` | string | No | `followers`, `repositories`, `joined` |
| `order` | string | No | `desc` (default) or `asc` |
| `per_page` | integer | No | Results per page (max 100, default 30) |
| `page` | integer | No | Page number (default 1) |

### Query Qualifiers

| Qualifier | Example | Description |
|-----------|---------|-------------|
| `in` | `in:login` | Search in `login`, `email`, `fullname` |
| `type` | `type:user` or `type:org` | Filter by type |
| `repos` | `repos:>10` | Filter by repo count |
| `followers` | `followers:>100` | Filter by follower count |
| `location` | `location:san francisco` | Filter by location |
| `language` | `language:python` | Filter by repos language |
| `created` | `created:>2024-01-01` | Filter by creation date |

### Query Examples
```
# Search by name
q=octocat in:fullname

# Search by location
q=location:beijing followers:>100

# Search by repos count
q=repos:>50 language:rust

# Search orgs
q=github type:org
```

---

## Search topics

### Endpoint
```
GET /search/topics?q={query}{&page,per_page,sort,order}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | The search query |
| `sort` | string | No | `stars` (default) |
| `order` | string | No | `desc` (default) or `asc` |
| `per_page` | integer | No | Results per page (max 100, default 30) |
| `page` | integer | No | Page number (default 1) |

### Query Qualifiers

| Qualifier | Example | Description |
|-----------|---------|-------------|
| `repositories` | `repositories:>10` | Filter by repo count |

---

## Search Syntax Tips

1. **Multiple qualifiers**: Combine with spaces
   ```
   q=label:bug state:open repo:owner/name
   ```

2. **Quoting**: Use quotes for multi-word terms
   ```
   q=in:title "fix bug"
   ```

3. **Negation**: Use `-` to exclude
   ```
   q=language:python -label:enhancement
   ```

4. **Boolean OR**: Use `|` for OR
   ```
   q=label:bug|enhancement
   ```

5. **Range queries**: Use `>`, `>=`, `<`, `<=`
   ```
   q=stars:>1000 forks:<50
   ```

6. **Date formats**: ISO 8601 or relative
   ```
   q=created:2024-01-01
   q=created:>2024-01
   q=pushed:>=2024-01-01T00:00:00Z
   ```

---

## Text Match Metadata

Request with `Accept: application/vnd.github+json` to get text matches:
```bash
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  "https://api.github.com/search/code?q=addClass+repo:jquery/jquery"
```

Response includes `text_matches` array highlighting matched terms.

---

## Reference

- [Official Documentation](https://docs.github.com/en/rest/search/search)
- API Version: 2022-11-28

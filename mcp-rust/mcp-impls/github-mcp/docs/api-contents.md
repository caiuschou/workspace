# Repository Contents API

## Overview

Use the REST API to create, modify, and delete Base64 encoded content in a repository.

### Base Endpoint
```
https://api.github.com/repos/{owner}/{repo}/contents/{path}
```

### Permissions Required
- **Read**: `Contents` repository permissions (read)
- **Write**: `Contents` repository permissions (write)
- **Workflows**: Additional `Workflows` repository permissions (write) for `.github/workflows` directory

---

## Get repository content

Gets the contents of a file or directory in a repository.

### Endpoint
```
GET /repos/{owner}/{repo}/contents/{path}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner` | string | Yes | The account owner of the repository (not case sensitive) |
| `repo` | string | Yes | The name of the repository without `.git` extension |
| `path` | string | Yes | The file path or directory path |
| `ref` | string | No | The name of the commit/branch/tag (default: repository's default branch) |

### Custom Media Types

| Media Type | Description |
|------------|-------------|
| `application/vnd.github.raw+json` | Returns raw file contents for files and symlinks |
| `application/vnd.github.html+json` | Returns file contents in HTML (rendered markup) |
| `application/vnd.github.object+json` | Returns contents in consistent object format |

### Response (File)
```json
{
  "type": "file",
  "encoding": "base64",
  "size": 5362,
  "name": "README.md",
  "path": "README.md",
  "content": "base64_encoded_content...",
  "sha": "3d21ec53a331a6f037a91c368710b99387d012c1",
  "url": "https://api.github.com/repos/octokit/octokit.rb/contents/README.md",
  "git_url": "https://api.github.com/repos/octokit/octokit.rb/git/blobs/...",
  "html_url": "https://github.com/octokit/octokit.rb/blob/master/README.md",
  "download_url": "https://raw.githubusercontent.com/octokit/octokit.rb/master/README.md"
}
```

### Response (Directory)
```json
[
  {
    "name": "src",
    "path": "src",
    "type": "dir",
    "url": "https://api.github.com/...",
    "git_url": "https://api.github.com/...",
    "html_url": "https://github.com/...",
    "_links": { "self": "...", "git": "...", "html": "..." }
  },
  {
    "name": "README.md",
    "path": "README.md",
    "type": "file",
    ...
  }
]
```

### File Size Limits
- **<= 1 MB**: All features supported
- **1-100 MB**: Only `raw` or `object` media types
- **> 100 MB**: Not supported (use Git Data API)

---

## Create or update file contents

Creates a new file or replaces an existing file in a repository.

### Endpoint
```
PUT /repos/{owner}/{repo}/contents/{path}
```

### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `message` | string | Yes | The commit message |
| `content` | string | Yes | The new file content, Base64 encoded |
| `sha` | string | No* | The blob SHA of the file being replaced (required for updates) |
| `branch` | string | No | The branch name (default: repository's default branch) |
| `committer` | object | No | The person that committed the file |
| `author` | object | No | The author of the file |

### Committer/Author Object
```json
{
  "name": "Monalisa Octocat",
  "email": "octocat@github.com",
  "date": "2014-11-07T22:01:45Z"
}
```

### Request Example
```bash
curl -L \
  -X PUT \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/contents/PATH \
  -d '{
    "message": "my commit message",
    "content": "bXkgbmV3IGZpbGUgY29udGVudHM="
  }'
```

### Response
```json
{
  "content": {
    "name": "hello.txt",
    "path": "notes/hello.txt",
    "sha": "95b966ae1c166bd92f8ae7d1c313e738c731dfc3",
    "size": 9,
    "type": "file"
  },
  "commit": {
    "sha": "7638417db6d59f3c431d3e1f261cc637155684cd",
    "message": "my commit message",
    "author": {
      "name": "Monalisa Octocat",
      "email": "octocat@github.com",
      "date": "2014-11-07T22:01:45Z"
    }
  }
}
```

### Status Codes
- `200`: File updated
- `201`: File created
- `404`: Resource not found
- `409`: Conflict
- `422`: Validation failed

---

## Delete a file

Deletes a file in a repository.

### Endpoint
```
DELETE /repos/{owner}/{repo}/contents/{path}
```

### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `message` | string | Yes | The commit message |
| `sha` | string | Yes | The blob SHA of the file being deleted |
| `branch` | string | No | The branch name |
| `committer` | object | No | The committer information |
| `author` | object | No | The author information |

### Request Example
```bash
curl -L \
  -X DELETE \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/contents/PATH \
  -d '{
    "message": "delete file",
    "sha": "329688480d39049927147c162b9d2deaf885005f"
  }'
```

---

## Get a repository README

Gets the preferred README for a repository.

### Endpoint
```
GET /repos/{owner}/{repo}/readme
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner` | string | Yes | The account owner |
| `repo` | string | Yes | The repository name |
| `ref` | string | No | The commit/branch/tag (default: default branch) |

### Custom Media Types
- `application/vnd.github.raw+json`: Raw file contents (default)
- `application/vnd.github.html+json`: README rendered to HTML

---

## Get a repository README for a directory

Gets the README from a repository directory.

### Endpoint
```
GET /repos/{owner}/{repo}/readme/{dir}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner` | string | Yes | The account owner |
| `repo` | string | Yes | The repository name |
| `dir` | string | Yes | The alternate path to look for a README file |
| `ref` | string | No | The commit/branch/tag |

---

## Download a repository archive

### Tar Archive
```
GET /repos/{owner}/{repo}/tarball/{ref}
```

### Zip Archive
```
GET /repos/{owner}/{repo}/zipball/{ref}
```

### Notes
- Returns a redirect (302) to the archive URL
- For private repos, links expire after 5 minutes
- Make sure HTTP client follows redirects

---

## Best Practices

1. **Check file existence** before updating - GET the file first to obtain the SHA
2. **Base64 encode** all file content when creating/updating
3. **Handle directories** - the API returns different structures for files vs directories
4. **Use appropriate media types** for your use case (raw, HTML, or object)
5. **Handle large files** - files > 100 MB require the Git Data API instead
6. **Lock mechanism** - don't use create/update and delete in parallel (will cause conflicts)

## Reference

- [Official Documentation](https://docs.github.com/en/rest/repos/contents)
- API Version: 2022-11-28

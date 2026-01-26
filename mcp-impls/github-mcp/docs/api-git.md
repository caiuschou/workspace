# Git Database API

## Overview

The Git Database API allows you to read and write Git objects (blobs, commits, references, tags, trees) to your Git database on GitHub.

### Base Endpoint
```
https://api.github.com/repos/{owner}/{repo}/git
```

---

## Git Blobs

A Git blob (binary large object) is the object type used to store the contents of each file in a repository.

### Create a blob

#### Endpoint
```
POST /repos/{owner}/{repo}/git/blobs
```

#### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `content` | string | Yes | The new blob's content |
| `encoding` | string | No | Encoding: `utf-8` or `base64` (default: `utf-8`) |

#### Request Example
```bash
curl -L \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/git/blobs \
  -d '{"content":"Content of the blob","encoding":"utf-8"}'
```

#### Response
```json
{
  "url": "https://api.github.com/repos/octocat/example/git/blobs/3a0f86fb8db8eea7ccbb9a95f325ddbedfb25e15",
  "sha": "3a0f86fb8db8eea7ccbb9a95f325ddbedfb25e15"
}
```

---

### Get a blob

#### Endpoint
```
GET /repos/{owner}/{repo}/git/blobs/{file_sha}
```

#### Media Types
- `application/vnd.github.raw+json`: Returns raw blob data
- `application/vnd.github+json`: Returns JSON with base64 encoded content (default)

#### Response
```json
{
  "content": "Q29udGVudCBvZiB0aGUgYmxvYg==",
  "encoding": "base64",
  "url": "https://api.github.com/repos/octocat/example/git/blobs/3a0f86fb8db8eea7ccbb9a95f325ddbedfb25e15",
  "sha": "3a0f86fb8db8eea7ccbb9a95f325ddbedfb25e15",
  "size": 19
}
```

**Note:** Supports blobs up to 100 megabytes.

---

## Git References (Refs)

A Git reference (git ref) is a file that contains a Git commit SHA-1 hash.

### List matching references

#### Endpoint
```
GET /repos/{owner}/{repo}/git/matching-refs/{ref}
```

The `:ref` must be formatted as `heads/<branch name>` for branches and `tags/<tag name>` for tags.

#### Response
```json
[
  {
    "ref": "refs/heads/feature-a",
    "node_id": "MDM6UmVmcmVmcy9oZWFkcy9mZWF0dXJlLWE=",
    "url": "https://api.github.com/repos/octocat/Hello-World/git/refs/heads/feature-a",
    "object": {
      "type": "commit",
      "sha": "aa218f56b14c9653891f9e74264a383fa43fefbd",
      "url": "https://api.github.com/repos/octocat/Hello-World/git/commits/aa218f56b14c9653891f9e74264a383fa43fefbd"
    }
  }
]
```

---

### Get a reference

#### Endpoint
```
GET /repos/{owner}/{repo}/git/ref/{ref}
```

#### Response
```json
{
  "ref": "refs/heads/featureA",
  "node_id": "MDM6UmVmcmVmcy9oZWFkcy9mZWF0dXJlQQ==",
  "url": "https://api.github.com/repos/octocat/Hello-World/git/refs/heads/featureA",
  "object": {
    "type": "commit",
    "sha": "aa218f56b14c9653891f9e74264a383fa43fefbd",
    "url": "https://api.github.com/repos/octocat/Hello-World/git/commits/aa218f56b14c9653891f9e74264a383fa43fefbd"
  }
}
```

---

### Create a reference

#### Endpoint
```
POST /repos/{owner}/{repo}/git/refs
```

#### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `ref` | string | Yes | The fully qualified reference (e.g., `refs/heads/master`) |
| `sha` | string | Yes | The SHA1 value for this reference |

#### Request Example
```bash
curl -L \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/git/refs \
  -d '{"ref":"refs/heads/featureA","sha":"aa218f56b14c9653891f9e74264a383fa43fefbd"}'
```

**Note:** Cannot create references for empty repositories.

---

### Update a reference

#### Endpoint
```
PATCH /repos/{owner}/{repo}/git/refs/{ref}
```

#### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sha` | string | Yes | The SHA1 value to set this reference to |
| `force` | boolean | No | Indicates whether to force the update (default: `false`) |

---

### Delete a reference

#### Endpoint
```
DELETE /repos/{owner}/{repo}/git/refs/{ref}
```

---

## Git Commits

### Create a commit

#### Endpoint
```
POST /repos/{owner}/{repo}/git/commits
```

#### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `message` | string | Yes | The commit message |
| `tree` | string | Yes | The SHA of the tree object |
| `parents` | array | Yes | Array of SHAs of the commits that were the parents of this commit |
| `author` | object | No | Author information |
| `committer` | object | No | Committer information |

#### Author/Committer Object
```json
{
  "name": "Monalisa Octocat",
  "email": "octocat@github.com",
  "date": "2014-11-07T22:01:45Z"
}
```

#### Request Example
```bash
curl -L \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/git/commits \
  -d '{"message":"message","tree":"tree_sha","parents":["parent_sha"]}'
```

---

### Get a commit

#### Endpoint
```
GET /repos/{owner}/{repo}/git/commits/{commit_sha}
```

---

## Git Tags

### Create a tag object

#### Endpoint
```
POST /repos/{owner}/{repo}/git/tags
```

#### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tag` | string | Yes | The tag's name |
| `message` | string | Yes | The tag message |
| `object` | string | Yes | The SHA of the git object this is tagging |
| `type` | string | Yes | The type of the object: `commit`, `tree`, or `blob` |
| `tagger` | object | No | Name and email of the person who created the tag |

---

### Get a tag

#### Endpoint
```
GET /repos/{owner}/{repo}/git/tags/{tag_sha}
```

---

## Git Trees

### Create a tree

#### Endpoint
```
POST /repos/{owner}/{repo}/git/trees
```

#### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tree` | array | Yes | Array of path/object SHA tuples |
| `base_tree` | string | No | The SHA of the tree you want to update with new data |

#### Tree Object
```json
{
  "path": "file.rb",
  "mode": "100644",
  "type": "blob",
  "sha": "44b4fc6d56897b048c772eb4087f854f79207a3"
}
```

| Mode | Description |
|------|-------------|
| `100644` | File (executable) |
| `100755` | File (executable) |
| `040000` | Directory (subtree) |
| `160000` | Submodule |

---

### Get a tree

#### Endpoint
```
GET /repos/{owner}/{repo}/git/trees/{tree_sha}
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `recursive` | boolean | No | Recursively get the tree |

---

## Permissions

| Action | Required Permission |
|--------|---------------------|
| Read Git objects | Contents (read) |
| Write Git objects | Contents (write) |
| Write Git objects in `.github/workflows` | Contents (write) + Workflows (write) |

---

## Reference

- [Git Blobs](https://docs.github.com/en/rest/git/blobs)
- [Git Commits](https://docs.github.com/en/rest/git/commits)
- [Git References](https://docs.github.com/en/rest/git/refs)
- [Git Tags](https://docs.github.com/en/rest/git/tags)
- [Git Trees](https://docs.github.com/en/rest/git/trees)
- API Version: 2022-11-28

# Releases API

## Overview

The Releases API allows you to create, modify, and delete releases. Releases are associated with Git tags.

### Base Endpoint
```
https://api.github.com/repos/{owner}/{repo}/releases
```

---

## List releases

### Endpoint
```
GET /repos/{owner}/{repo}/releases
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `per_page` | integer | No | Results per page (max 100, default 30) |
| `page` | integer | No | Page number (default 1) |

**Note:** This returns published releases. Draft releases are only visible to users with push access.

### Response
```json
[
  {
    "url": "https://api.github.com/repos/octocat/Hello-World/releases/1",
    "html_url": "https://github.com/octocat/Hello-World/releases/v1.0.0",
    "assets_url": "https://api.github.com/repos/octocat/Hello-World/releases/1/assets",
    "upload_url": "https://uploads.github.com/repos/octocat/Hello-World/releases/1/assets{?name,label}",
    "tarball_url": "https://api.github.com/repos/octocat/Hello-World/tarball/v1.0.0",
    "zipball_url": "https://api.github.com/repos/octocat/Hello-World/zipball/v1.0.0",
    "id": 1,
    "node_id": "MDc6UmVsZWFzZTE=",
    "tag_name": "v1.0.0",
    "target_commitish": "master",
    "name": "v1.0.0",
    "body": "Description of the release",
    "draft": false,
    "prerelease": false,
    "immutable": false,
    "created_at": "2013-02-27T19:35:32Z",
    "published_at": "2013-02-27T19:35:32Z",
    "author": {
      "login": "octocat",
      "id": 1,
      "avatar_url": "https://github.com/images/error/octocat_happy.gif"
    },
    "assets": [
      {
        "url": "https://api.github.com/repos/octocat/Hello-World/releases/assets/1",
        "browser_download_url": "https://github.com/octocat/Hello-World/releases/download/v1.0.0/example.zip",
        "id": 1,
        "name": "example.zip",
        "label": "short description",
        "state": "uploaded",
        "content_type": "application/zip",
        "size": 1024,
        "download_count": 42,
        "created_at": "2013-02-27T19:35:32Z"
      }
    ]
  }
]
```

---

## Create a release

### Endpoint
```
POST /repos/{owner}/{repo}/releases
```

### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tag_name` | string | Yes | The name of the tag |
| `target_commitish` | string | No | Specifies the commitish value for the tag |
| `name` | string | No | The name of the release |
| `body` | string | No | Text describing the contents of the tag |
| `draft` | boolean | No | `true` to create a draft (default: `false`) |
| `prerelease` | boolean | No | `true` to identify as a prerelease (default: `false`) |
| `discussion_category_name` | string | No | Discussion category to link |
| `generate_release_notes` | boolean | No | Auto-generate name and body (default: `false`) |
| `make_latest` | string | No | Set as latest: `true`, `false`, or `legacy` (default: `true`) |

### Request Example
```bash
curl -L \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/releases \
  -d '{
    "tag_name": "v1.0.0",
    "target_commitish": "master",
    "name": "v1.0.0",
    "body": "Description of the release",
    "draft": false,
    "prerelease": false
  }'
```

---

## Get the latest release

### Endpoint
```
GET /repos/{owner}/{repo}/releases/latest
```

Returns the most recent non-prerelease, non-draft release sorted by `created_at`.

---

## Get a release by tag name

### Endpoint
```
GET /repos/{owner}/{repo}/releases/tags/{tag}
```

---

## Get a release

### Endpoint
```
GET /repos/{owner}/{repo}/releases/{release_id}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `release_id` | integer | Yes | The unique identifier of the release |

**Note:** Returns an `upload_url` for uploading release assets.

---

## Update a release

### Endpoint
```
PATCH /repos/{owner}/{repo}/releases/{release_id}
```

### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tag_name` | string | No | The name of the tag |
| `target_commitish` | string | No | The commitish value for the tag |
| `name` | string | No | The name of the release |
| `body` | string | No | Text describing the release |
| `draft` | boolean | No | `true` makes a draft, `false` publishes |
| `prerelease` | boolean | No | `true` for prerelease, `false` for full release |
| `make_latest` | string | No | Set as latest: `true`, `false`, or `legacy` |

---

## Delete a release

### Endpoint
```
DELETE /repos/{owner}/{repo}/releases/{release_id}
```

---

## Generate release notes

### Endpoint
```
POST /repos/{owner}/{repo}/releases/generate-notes
```

Generates a name and body describing a release. The generated notes are not saved.

### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tag_name` | string | Yes | The tag name for the release |
| `target_commitish` | string | No | The target commitish value |
| `previous_tag_name` | string | No | The previous tag name for the range |
| `configuration_file_path` | string | No | Path to configuration file |

### Response
```json
{
  "name": "Release v1.0.0 is now available!",
  "body": "## Changes in Release v1.0.0 ... ## Contributors @monalisa"
}
```

---

## Release Assets API

### List release assets
```
GET /repos/{owner}/{repo}/releases/{release_id}/assets
```

### Get a release asset
```
GET /repos/{owner}/{repo}/releases/assets/{asset_id}
```

### Update a release asset
```
PATCH /repos/{owner}/{repo}/releases/assets/{asset_id}
```

#### Request Body
```json
{
  "name": "new name",
  "label": "new label"
}
```

### Delete a release asset
```
DELETE /repos/{owner}/{repo}/releases/assets/{asset_id}
```

### Upload a release asset

#### Endpoint
```
POST https://uploads.github.com/repos/{owner}/{repo}/releases/{release_id}/assets?name=foo.zip
```

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | The file name of the asset |
| `label` | string | No | Short description of the asset |

#### Request Example
```bash
curl -L \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/zip" \
  --data-binary @file.zip \
  "https://uploads.github.com/repos/OWNER/REPO/releases/RELEASE_ID/assets?name=file.zip&label=short+description"
```

---

## Permissions

| Action | Required Permission |
|--------|---------------------|
| Read releases | Contents (read) |
| Create/update/delete releases | Contents (write) |
| Create releases in `.github/workflows` | Contents (write) + Workflows (write) |

---

## Reference

- [Official Documentation](https://docs.github.com/en/rest/releases/releases)
- API Version: 2022-11-28

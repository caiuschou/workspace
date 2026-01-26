# Webhooks API

## Overview

Webhooks allow you to receive HTTP notifications when certain events occur in a repository.

### Base Endpoint
```
https://api.github.com/repos/{owner}/{repo}/hooks
```

### Webhook Events

Webhooks can be triggered by various events:
- `push` - Git push to a repository
- `create` - Branch or tag created
- `delete` - Branch or tag deleted
- `issues` - Issue opened/edited/deleted
- `issue_comment` - Issue comment created/edited/deleted
- `pull_request` - PR opened/edited/closed/synchronized
- `pull_request_review` - PR review submitted/edited/dismissed
- `release` - Release published
- `watch` - User starred/unstarred repository
- `fork` - Repository forked
- `ping` - Ping event

---

## Repository Webhooks

### List webhooks

#### Endpoint
```
GET /repos/{owner}/{repo}/hooks
```

#### Response
```json
[
  {
    "id": 1,
    "url": "https://api.github.com/repos/octocat/Hello-World/hooks/1",
    "type": "Repository",
    "name": "web",
    "active": true,
    "events": ["push", "pull_request"],
    "config": {
      "content_type": "json",
      "insecure_ssl": "false",
      "url": "http://example.com/webhook",
      "secret": "********"
    },
    "updated_at": "2014-06-26T07:14:43Z",
    "created_at": "2014-06-26T07:14:43Z"
  }
]
```

---

### Create a webhook

#### Endpoint
```
POST /repos/{owner}/{repo}/hooks
```

#### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Must be "web" |
| `config` | object | Yes | Key-value pair configuration |
| `events` | array | No | Events to trigger the webhook (default: `["push"]`) |
| `active` | boolean | No | Whether the webhook is active (default: `true`) |

#### Config Object

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url` | string | Yes | The URL to which the payloads will be delivered |
| `content_type` | string | No | The media type: `json` or `form` |
| `secret` | string | No | If provided, the secret will be used as the `X-Hub-Signature` header |
| `insecure_ssl` | string | No | Determines whether the SSL certificate is verified |

#### Request Example
```bash
curl -L \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/OWNER/REPO/hooks \
  -d '{
    "name": "web",
    "active": true,
    "events": ["push", "pull_request"],
    "config": {
      "url": "http://example.com/webhook",
      "content_type": "json",
      "secret": "my_secret"
    }
  }'
```

---

### Get a webhook

#### Endpoint
```
GET /repos/{owner}/{repo}/hooks/{hook_id}
```

---

### Update a webhook

#### Endpoint
```
PATCH /repos/{owner}/{repo}/hooks/{hook_id}
```

#### Request Body

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `config` | object | No | Key-value pair configuration |
| `events` | array | No | Events to trigger the webhook |
| `active` | boolean | No | Whether the webhook is active |
| `add_events` | array | No | Events to add to the webhook |
| `remove_events` | array | No | Events to remove from the webhook |

---

### Delete a webhook

#### Endpoint
```
DELETE /repos/{owner}/{repo}/hooks/{hook_id}
```

---

### Ping a webhook

#### Endpoint
```
POST /repos/{owner}/{repo}/hooks/{hook_id}/pings
```

Triggers a ping event to be sent to the webhook.

---

## Webhook Configurations

### Get webhook configuration

#### Endpoint
```
GET /repos/{owner}/{repo}/hooks/{hook_id}/config
```

### Update webhook configuration

#### Endpoint
```
PATCH /repos/{owner}/{repo}/hooks/{hook_id}/config
```

---

## Webhook Deliveries

### List deliveries

#### Endpoint
```
GET /repos/{owner}/{repo}/hooks/{hook_id}/deliveries
```

### Get a delivery

#### Endpoint
```
GET /repos/{owner}/{repo}/hooks/{hook_id}/deliveries/{delivery_id}
```

### Redeliver a payload

#### Endpoint
```
POST /repos/{owner}/{repo}/hooks/{hook_id}/deliveries/{delivery_id}/attempts
```

---

## Organization Webhooks

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

## Webhook Payloads

### Push Event Payload

```json
{
  "ref": "refs/heads/main",
  "repository": {
    "id": 1296269,
    "name": "Hello-World",
    "full_name": "octocat/Hello-World",
    "private": false,
    "owner": {
      "login": "octocat",
      "id": 1
    }
  },
  "pusher": {
    "login": "octocat",
    "email": "octocat@github.com"
  },
  "sender": {
    "login": "octocat",
    "id": 1
  }
}
```

### Pull Request Event Payload

```json
{
  "action": "opened",
  "number": 1,
  "pull_request": {
    "id": 1,
    "number": 1,
    "state": "open",
    "title": "new-feature",
    "user": {
      "login": "octocat"
    },
    "head": {
      "ref": "new-feature",
      "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e"
    },
    "base": {
      "ref": "main",
      "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e"
    }
  },
  "repository": {
    "id": 1296269
  },
  "sender": {
    "login": "octocat"
  }
}
```

---

## Webhook Headers

| Header | Description |
|--------|-------------|
| `X-GitHub-Event` | The name of the event that triggered the delivery |
| `X-Hub-Signature-256` | The HMAC SHA256 signature of the payload |
| `X-GitHub-Delivery` | Unique delivery identifier |
| `X-GitHub-Enterprise` | The GitHub Enterprise hostname |
| `X-GitHub-Delivery-Timestamp` | Timestamp of when the delivery was initiated |

---

## Permissions

| Action | Required Permission |
|--------|---------------------|
| Read webhooks | Admin or Meta (read) |
| Create/update/delete webhooks | Admin or Meta (write) |

---

## Best Practices

1. **Use HTTPS** for webhook URLs
2. **Verify signatures** using `X-Hub-Signature-256` header
3. **Handle timeouts** - webhooks timeout after 10 seconds
4. **Retry failed deliveries** - GitHub will retry up to 3 times
5. **Use appropriate content type** - `json` is recommended
6. **Keep secrets secure** - don't expose them in code

---

## Reference

- [Official Documentation](https://docs.github.com/en/rest/webhooks)
- API Version: 2022-11-28

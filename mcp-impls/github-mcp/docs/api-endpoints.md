# GitHub REST API Endpoints Reference

Complete list of REST API endpoints available for fine-grained personal access tokens.

**API Version:** 2022-11-28 (latest)
**Documentation Date:** 2025-01-21

---

## Actions

### Artifacts
```
GET /repos/{owner}/{repo}/actions/artifacts
GET /repos/{owner}/{repo}/actions/artifacts/{artifact_id}
DELETE /repos/{owner}/{repo}/actions/artifacts/{artifact_id}
GET /repos/{owner}/{repo}/actions/artifacts/{artifact_id}/{archive_format}
```

### Cache
```
GET /repos/{owner}/{repo}/actions/cache/usage
GET /repos/{owner}/{repo}/actions/caches
DELETE /repos/{owner}/{repo}/actions/caches
DELETE /repos/{owner}/{repo}/actions/caches/{cache_id}
GET /orgs/{org}/actions/cache/usage
GET /orgs/{org}/actions/cache/usage-by-repository
```

### OIDC
```
GET /repos/{owner}/{repo}/actions/oidc/customization/sub
PUT /repos/{owner}/{repo}/actions/oidc/customization/sub
GET /orgs/{org}/actions/oidc/customization/sub
PUT /orgs/{org}/actions/oidc/customization/sub
```

### Permissions
```
GET /repos/{owner}/{repo}/actions/permissions
PUT /repos/{owner}/{repo}/actions/permissions
GET /repos/{owner}/{repo}/actions/permissions/access
PUT /repos/{owner}/{repo}/actions/permissions/access
GET /repos/{owner}/{repo}/actions/permissions/selected-actions
PUT /repos/{owner}/{repo}/actions/permissions/selected-actions
GET /repos/{owner}/{repo}/actions/permissions/workflow
PUT /repos/{owner}/{repo}/actions/permissions/workflow
```

### Runners (Self-hosted)
```
GET /repos/{owner}/{repo}/actions/runners
GET /repos/{owner}/{repo}/actions/runners/downloads
POST /repos/{owner}/{repo}/actions/runners/generate-jitconfig
POST /repos/{owner}/{repo}/actions/runners/registration-token
POST /repos/{owner}/{repo}/actions/runners/remove-token
GET /repos/{owner}/{repo}/actions/runners/{runner_id}
DELETE /repos/{owner}/{repo}/actions/runners/{runner_id}
GET /repos/{owner}/{repo}/actions/runners/{runner_id}/labels
POST /repos/{owner}/{repo}/actions/runners/{runner_id}/labels
PUT /repos/{owner}/{repo}/actions/runners/{runner_id}/labels
DELETE /repos/{owner}/{repo}/actions/runners/{runner_id}/labels
DELETE /repos/{owner}/{repo}/actions/runners/{runner_id}/labels/{name}
```

### Secrets
```
GET /repos/{owner}/{repo}/actions/secrets
GET /repos/{owner}/{repo}/actions/secrets/public-key
GET /repos/{owner}/{repo}/actions/secrets/{secret_name}
PUT /repos/{owner}/{repo}/actions/secrets/{secret_name}
DELETE /repos/{owner}/{repo}/actions/secrets/{secret_name}
GET /repos/{owner}/{repo}/environments/{environment_name}/secrets
GET /repos/{owner}/{repo}/environments/{environment_name}/secrets/public-key
GET /repos/{owner}/{repo}/environments/{environment_name}/secrets/{secret_name}
PUT /repos/{owner}/{repo}/environments/{environment_name}/secrets/{secret_name}
DELETE /repos/{owner}/{repo}/environments/{environment_name}/secrets/{secret_name}
```

### Variables
```
GET /repos/{owner}/{repo}/actions/variables
POST /repos/{owner}/{repo}/actions/variables
GET /repos/{owner}/{repo}/actions/variables/{name}
PATCH /repos/{owner}/{repo}/actions/variables/{name}
DELETE /repos/{owner}/{repo}/actions/variables/{name}
GET /repos/{owner}/{repo}/environments/{environment_name}/variables
POST /repos/{owner}/{repo}/environments/{environment_name}/variables
GET /repos/{owner}/{repo}/environments/{environment_name}/variables/{name}
PATCH /repos/{owner}/{repo}/environments/{environment_name}/variables/{name}
DELETE /repos/{owner}/{repo}/environments/{environment_name}/variables/{name}
```

### Workflow Runs
```
GET /repos/{owner}/{repo}/actions/runs
GET /repos/{owner}/{repo}/actions/runs/{run_id}
DELETE /repos/{owner}/{repo}/actions/runs/{run_id}
GET /repos/{owner}/{repo}/actions/runs/{run_id}/approvals
POST /repos/{owner}/{repo}/actions/runs/{run_id}/approve
GET /repos/{owner}/{repo}/actions/runs/{run_id}/artifacts
GET /repos/{owner}/{repo}/actions/runs/{run_id}/attempts/{attempt_number}
GET /repos/{owner}/{repo}/actions/runs/{run_id}/attempts/{attempt_number}/jobs
GET /repos/{owner}/{repo}/actions/runs/{run_id}/attempts/{attempt_number}/logs
POST /repos/{owner}/{repo}/actions/runs/{run_id}/cancel
POST /repos/{owner}/{repo}/actions/runs/{run_id}/force-cancel
GET /repos/{owner}/{repo}/actions/runs/{run_id}/jobs
GET /repos/{owner}/{repo}/actions/runs/{run_id}/logs
DELETE /repos/{owner}/{repo}/actions/runs/{run_id}/logs
GET /repos/{owner}/{repo}/actions/runs/{run_id}/pending_deployments
POST /repos/{owner}/{repo}/actions/runs/{run_id}/pending_deployments
POST /repos/{owner}/{repo}/actions/runs/{run_id}/rerun
POST /repos/{owner}/{repo}/actions/runs/{run_id}/rerun-failed-jobs
GET /repos/{owner}/{repo}/actions/runs/{run_id}/timing
```

### Workflow Jobs
```
GET /repos/{owner}/{repo}/actions/jobs/{job_id}
GET /repos/{owner}/{repo}/actions/jobs/{job_id}/logs
POST /repos/{owner}/{repo}/actions/jobs/{job_id}/rerun
```

### Workflows
```
GET /repos/{owner}/{repo}/actions/workflows
GET /repos/{owner}/{repo}/actions/workflows/{workflow_id}
PUT /repos/{owner}/{repo}/actions/workflows/{workflow_id}/disable
POST /repos/{owner}/{repo}/actions/workflows/{workflow_id}/dispatches
PUT /repos/{owner}/{repo}/actions/workflows/{workflow_id}/enable
GET /repos/{owner}/{repo}/actions/workflows/{workflow_id}/runs
GET /repos/{owner}/{repo}/actions/workflows/{workflow_id}/timing
```

---

## Activity

```
GET /repos/{owner}/{repo}/events
GET /repos/{owner}/{repo}/stargazers
GET /repos/{owner}/{repo}/subscribers
GET /user/starred
GET /user/starred/{owner}/{repo}
PUT /user/starred/{owner}/{repo}
DELETE /user/starred/{owner}/{repo}
GET /user/subscriptions
GET /users/{username}/events/orgs/{org}
GET /users/{username}/starred
GET /users/{username}/subscriptions
```

---

## Billing

```
GET /orgs/{org}/settings/billing/actions
GET /orgs/{org}/settings/billing/packages
GET /orgs/{org}/settings/billing/shared-storage
GET /users/{username}/settings/billing/actions
GET /users/{username}/settings/billing/packages
GET /users/{username}/settings/billing/shared-storage
```

---

## Branches

```
GET /repos/{owner}/{repo}/branches
GET /repos/{owner}/{repo}/branches/{branch}
GET /repos/{owner}/{repo}/branches/{branch}/protection
PUT /repos/{owner}/{repo}/branches/{branch}/protection
DELETE /repos/{owner}/{repo}/branches/{branch}/protection
GET /repos/{owner}/{repo}/branches/{branch}/protection/enforce_admins
POST /repos/{owner}/{repo}/branches/{branch}/protection/enforce_admins
DELETE /repos/{owner}/{repo}/branches/{branch}/protection/enforce_admins
GET /repos/{owner}/{repo}/branches/{branch}/protection/required_pull_request_reviews
PATCH /repos/{owner}/{repo}/branches/{branch}/protection/required_pull_request_reviews
DELETE /repos/{owner}/{repo}/branches/{branch}/protection/required_pull_request_reviews
GET /repos/{owner}/{repo}/branches/{branch}/protection/required_signatures
POST /repos/{owner}/{repo}/branches/{branch}/protection/required_signatures
DELETE /repos/{owner}/{repo}/branches/{branch}/protection/required_signatures
GET /repos/{owner}/{repo}/branches/{branch}/protection/required_status_checks
PATCH /repos/{owner}/{repo}/branches/{branch}/protection/required_status_checks
DELETE /repos/{owner}/{repo}/branches/{branch}/protection/required_status_checks
GET /repos/{owner}/{repo}/branches/{branch}/protection/required_status_checks/contexts
POST /repos/{owner}/{repo}/branches/{branch}/protection/required_status_checks/contexts
PUT /repos/{owner}/{repo}/branches/{branch}/protection/required_status_checks/contexts
DELETE /repos/{owner}/{repo}/branches/{branch}/protection/required_status_checks/contexts
GET /repos/{owner}/{repo}/branches/{branch}/protection/restrictions
DELETE /repos/{owner}/{repo}/branches/{branch}/protection/restrictions
GET /repos/{owner}/{repo}/branches/{branch}/protection/restrictions/apps
POST /repos/{owner}/{repo}/branches/{branch}/protection/restrictions/apps
PUT /repos/{owner}/{repo}/branches/{branch}/protection/restrictions/apps
DELETE /repos/{owner}/{repo}/branches/{branch}/protection/restrictions/apps
GET /repos/{owner}/{repo}/branches/{branch}/protection/restrictions/teams
POST /repos/{owner}/{repo}/branches/{branch}/protection/restrictions/teams
PUT /repos/{owner}/{repo}/branches/{branch}/protection/restrictions/teams
DELETE /repos/{owner}/{repo}/branches/{branch}/protection/restrictions/teams
GET /repos/{owner}/{repo}/branches/{branch}/protection/restrictions/users
POST /repos/{owner}/{repo}/branches/{branch}/protection/restrictions/users
PUT /repos/{owner}/{repo}/branches/{branch}/protection/restrictions/users
DELETE /repos/{owner}/{repo}/branches/{branch}/protection/restrictions/users
POST /repos/{owner}/{repo}/branches/{branch}/rename
POST /repos/{owner}/{repo}/merge-upstream
POST /repos/{owner}/{repo}/merges
```

---

## Checks

```
POST /repos/{owner}/{repo}/check-runs
GET /repos/{owner}/{repo}/check-runs/{check_run_id}
PATCH /repos/{owner}/{repo}/check-runs/{check_run_id}
GET /repos/{owner}/{repo}/check-runs/{check_run_id}/annotations
POST /repos/{owner}/{repo}/check-runs/{check_run_id}/rerequest
POST /repos/{owner}/{repo}/check-suites
PATCH /repos/{owner}/{repo}/check-suites/preferences
GET /repos/{owner}/{repo}/check-suites/{check_suite_id}
GET /repos/{owner}/{repo}/check-suites/{check_suite_id}/check-runs
POST /repos/{owner}/{repo}/check-suites/{check_suite_id}/rerequest
GET /repos/{owner}/{repo}/commits/{ref}/check-runs
GET /repos/{owner}/{repo}/commits/{ref}/check-suites
```

---

## Code Scanning

```
GET /orgs/{org}/code-scanning/alerts
GET /repos/{owner}/{repo}/code-scanning/alerts
GET /repos/{owner}/{repo}/code-scanning/alerts/{alert_number}
PATCH /repos/{owner}/{repo}/code-scanning/alerts/{alert_number}
GET /repos/{owner}/{repo}/code-scanning/alerts/{alert_number}/instances
GET /repos/{owner}/{repo}/code-scanning/analyses
GET /repos/{owner}/{repo}/code-scanning/analyses/{analysis_id}
DELETE /repos/{owner}/{repo}/code-scanning/analyses/{analysis_id}
GET /repos/{owner}/{repo}/code-scanning/codeql/databases
GET /repos/{owner}/{repo}/code-scanning/codeql/databases/{language}
PATCH /repos/{owner}/{repo}/code-scanning/default-setup
POST /repos/{owner}/{repo}/code-scanning/sarifs
GET /repos/{owner}/{repo}/code-scanning/sarifs/{sarif_id}
```

---

## Codespaces

```
GET /orgs/{org}/codespaces
PUT /orgs/{org}/codespaces/access
POST /orgs/{org}/codespaces/access/selected_users
DELETE /orgs/{org}/codespaces/access/selected_users
GET /orgs/{org}/codespaces/secrets
GET /orgs/{org}/codespaces/secrets/public-key
GET /orgs/{org}/codespaces/secrets/{secret_name}
PUT /orgs/{org}/codespaces/secrets/{secret_name}
DELETE /orgs/{org}/codespaces/secrets/{secret_name}
GET /orgs/{org}/members/{username}/codespaces
DELETE /orgs/{org}/members/{username}/codespaces/{codespace_name}
POST /orgs/{org}/members/{username}/codespaces/{codespace_name}/stop
GET /repos/{owner}/{repo}/codespaces
POST /repos/{owner}/{repo}/codespaces
GET /repos/{owner}/{repo}/codespaces/devcontainers
GET /repos/{owner}/{repo}/codespaces/machines
GET /repos/{owner}/{repo}/codespaces/new
GET /repos/{owner}/{repo}/codespaces/permissions_check
GET /repos/{owner}/{repo}/codespaces/secrets
GET /repos/{owner}/{repo}/codespaces/secrets/public-key
GET /repos/{owner}/{repo}/codespaces/secrets/{secret_name}
PUT /repos/{owner}/{repo}/codespaces/secrets/{secret_name}
DELETE /repos/{owner}/{repo}/codespaces/secrets/{secret_name}
POST /repos/{owner}/{repo}/pulls/{pull_number}/codespaces
GET /user/codespaces
POST /user/codespaces
GET /user/codespaces/secrets
GET /user/codespaces/secrets/public-key
GET /user/codespaces/secrets/{secret_name}
PUT /user/codespaces/secrets/{secret_name}
DELETE /user/codespaces/secrets/{secret_name}
GET /user/codespaces/{codespace_name}
PATCH /user/codespaces/{codespace_name}
DELETE /user/codespaces/{codespace_name}
POST /user/codespaces/{codespace_name}/exports
GET /user/codespaces/{codespace_name}/exports/{export_id}
GET /user/codespaces/{codespace_name}/machines
POST /user/codespaces/{codespace_name}/publish
POST /user/codespaces/{codespace_name}/start
POST /user/codespaces/{codespace_name}/stop
```

---

## Collaborators

```
GET /repos/{owner}/{repo}/collaborators
GET /repos/{owner}/{repo}/collaborators/{username}
PUT /repos/{owner}/{repo}/collaborators/{username}
DELETE /repos/{owner}/{repo}/collaborators/{username}
GET /repos/{owner}/{repo}/collaborators/{username}/permission
GET /repos/{owner}/{repo}/invitations
PATCH /repos/{owner}/{repo}/invitations/{invitation_id}
DELETE /repos/{owner}/{repo}/invitations/{invitation_id}
GET /user/repository_invitations
DELETE /user/repository_invitations/{invitation_id}
```

---

## Commits

```
GET /repos/{owner}/{repo}/comments
GET /repos/{owner}/{repo}/comments/{comment_id}
PATCH /repos/{owner}/{repo}/comments/{comment_id}
DELETE /repos/{owner}/{repo}/comments/{comment_id}
GET /repos/{owner}/{repo}/commits
GET /repos/{owner}/{repo}/commits/{commit_sha}/branches-where-head
GET /repos/{owner}/{repo}/commits/{commit_sha}/comments
POST /repos/{owner}/{repo}/commits/{commit_sha}/comments
GET /repos/{owner}/{repo}/commits/{commit_sha}/pulls
GET /repos/{owner}/{repo}/commits/{ref}
GET /repos/{owner}/{repo}/commits/{ref}/status
GET /repos/{owner}/{repo}/commits/{ref}/statuses
GET /repos/{owner}/{repo}/compare/{basehead}
POST /repos/{owner}/{repo}/statuses/{sha}
```

---

## Deploy Keys

```
GET /repos/{owner}/{repo}/keys
POST /repos/{owner}/{repo}/keys
GET /repos/{owner}/{repo}/keys/{key_id}
DELETE /repos/{owner}/{repo}/keys/{key_id}
```

---

## Deployments

```
GET /repos/{owner}/{repo}/deployments
POST /repos/{owner}/{repo}/deployments
GET /repos/{owner}/{repo}/deployments/{deployment_id}
DELETE /repos/{owner}/{repo}/deployments/{deployment_id}
GET /repos/{owner}/{repo}/deployments/{deployment_id}/statuses
POST /repos/{owner}/{repo}/deployments/{deployment_id}/statuses
GET /repos/{owner}/{repo}/deployments/{deployment_id}/statuses/{status_id}
GET /repos/{owner}/{repo}/environments
GET /repos/{owner}/{repo}/environments/{environment_name}
PUT /repos/{owner}/{repo}/environments/{environment_name}
DELETE /repos/{owner}/{repo}/environments/{environment_name}
GET /repos/{owner}/{repo}/environments/{environment_name}/deployment-branch-policies
POST /repos/{owner}/{repo}/environments/{environment_name}/deployment-branch-policies
GET /repos/{owner}/{repo}/environments/{environment_name}/deployment-branch-policies/{branch_policy_id}
PUT /repos/{owner}/{repo}/environments/{environment_name}/deployment-branch-policies/{branch_policy_id}
DELETE /repos/{owner}/{repo}/environments/{environment_name}/deployment-branch-policies/{branch_policy_id}
GET /repos/{owner}/{repo}/environments/{environment_name}/deployment_protection_rules
POST /repos/{owner}/{repo}/environments/{environment_name}/deployment_protection_rules
GET /repos/{owner}/{repo}/environments/{environment_name}/deployment_protection_rules/apps
GET /repos/{owner}/{repo}/environments/{environment_name}/deployment_protection_rules/{protection_rule_id}
DELETE /repos/{owner}/{repo}/environments/{environment_name}/deployment_protection_rules/{protection_rule_id}
```

---

## Emojis

```
GET /emojis
```

---

## Gists

```
GET /gists
POST /gists
GET /gists/public
GET /gists/starred
GET /gists/{gist_id}
PATCH /gists/{gist_id}
DELETE /gists/{gist_id}
GET /gists/{gist_id}/comments
POST /gists/{gist_id}/comments
GET /gists/{gist_id}/comments/{comment_id}
PATCH /gists/{gist_id}/comments/{comment_id}
DELETE /gists/{gist_id}/comments/{comment_id}
GET /gists/{gist_id}/commits
GET /gists/{gist_id}/forks
POST /gists/{gist_id}/forks
GET /gists/{gist_id}/star
PUT /gists/{gist_id}/star
DELETE /gists/{gist_id}/star
GET /gists/{gist_id}/{sha}
GET /users/{username}/gists
```

---

## Git (Database)

```
POST /repos/{owner}/{repo}/git/blobs
GET /repos/{owner}/{repo}/git/blobs/{file_sha}
POST /repos/{owner}/{repo}/git/commits
GET /repos/{owner}/{repo}/git/commits/{commit_sha}
GET /repos/{owner}/{repo}/git/matching-refs/{ref}
GET /repos/{owner}/{repo}/git/ref/{ref}
POST /repos/{owner}/{repo}/git/refs
PATCH /repos/{owner}/{repo}/git/refs/{ref}
DELETE /repos/{owner}/{repo}/git/refs/{ref}
POST /repos/{owner}/{repo}/git/tags
GET /repos/{owner}/{repo}/git/tags/{tag_sha}
POST /repos/{owner}/{repo}/git/trees
GET /repos/{owner}/{repo}/git/trees/{tree_sha}
```

---

## Gitignore

```
GET /gitignore/templates
GET /gitignore/templates/{name}
```

---

## Interactions

```
GET /orgs/{org}/interaction-limits
PUT /orgs/{org}/interaction-limits
DELETE /orgs/{org}/interaction-limits
GET /repos/{owner}/{repo}/interaction-limits
PUT /repos/{owner}/{repo}/interaction-limits
DELETE /repos/{owner}/{repo}/interaction-limits
GET /user/interaction-limits
PUT /user/interaction-limits
DELETE /user/interaction-limits
```

---

## Issues

### Issue Operations
```
GET /repos/{owner}/{repo}/assignees
GET /repos/{owner}/{repo}/assignees/{assignee}
GET /repos/{owner}/{repo}/issues
POST /repos/{owner}/{repo}/issues
GET /repos/{owner}/{repo}/issues/comments
GET /repos/{owner}/{repo}/issues/comments/{comment_id}
PATCH /repos/{owner}/{repo}/issues/comments/{comment_id}
DELETE /repos/{owner}/{repo}/issues/comments/{comment_id}
GET /repos/{owner}/{repo}/issues/events
GET /repos/{owner}/{repo}/issues/events/{event_id}
GET /repos/{owner}/{repo}/issues/{issue_number}
PATCH /repos/{owner}/{repo}/issues/{issue_number}
POST /repos/{owner}/{repo}/issues/{issue_number}/assignees
DELETE /repos/{owner}/{repo}/issues/{issue_number}/assignees
GET /repos/{owner}/{repo}/issues/{issue_number}/assignees/{assignee}
GET /repos/{owner}/{repo}/issues/{issue_number}/comments
POST /repos/{owner}/{repo}/issues/{issue_number}/comments
GET /repos/{owner}/{repo}/issues/{issue_number}/events
GET /repos/{owner}/{repo}/issues/{issue_number}/labels
POST /repos/{owner}/{repo}/issues/{issue_number}/labels
PUT /repos/{owner}/{repo}/issues/{issue_number}/labels
DELETE /repos/{owner}/{repo}/issues/{issue_number}/labels
DELETE /repos/{owner}/{repo}/issues/{issue_number}/labels/{name}
PUT /repos/{owner}/{repo}/issues/{issue_number}/lock
DELETE /repos/{owner}/{repo}/issues/{issue_number}/lock
GET /repos/{owner}/{repo}/issues/{issue_number}/timeline
GET /user/issues
```

### Labels
```
GET /repos/{owner}/{repo}/labels
POST /repos/{owner}/{repo}/labels
GET /repos/{owner}/{repo}/labels/{name}
PATCH /repos/{owner}/{repo}/labels/{name}
DELETE /repos/{owner}/{repo}/labels/{name}
```

### Milestones
```
GET /repos/{owner}/{repo}/milestones
POST /repos/{owner}/{repo}/milestones
GET /repos/{owner}/{repo}/milestones/{milestone_number}
PATCH /repos/{owner}/{repo}/milestones/{milestone_number}
DELETE /repos/{owner}/{repo}/milestones/{milestone_number}
GET /repos/{owner}/{repo}/milestones/{milestone_number}/labels
```

---

## Licenses

```
GET /licenses
GET /licenses/{license}
GET /repos/{owner}/{repo}/license
```

---

## Markdown

```
POST /markdown
POST /markdown/raw
```

---

## Meta

```
GET /
GET /meta
GET /octocat
GET /versions
GET /zen
```

---

## Metrics

```
GET /repos/{owner}/{repo}/community/profile
GET /repos/{owner}/{repo}/stats/code_frequency
GET /repos/{owner}/{repo}/stats/commit_activity
GET /repos/{owner}/{repo}/stats/contributors
GET /repos/{owner}/{repo}/stats/participation
GET /repos/{owner}/{repo}/stats/punch_card
GET /repos/{owner}/{repo}/traffic/clones
GET /repos/{owner}/{repo}/traffic/popular/paths
GET /repos/{owner}/{repo}/traffic/popular/referrers
GET /repos/{owner}/{repo}/traffic/views
```

---

## Organizations

```
PATCH /orgs/{org}
DELETE /orgs/{org}
GET /orgs/{org}/blocks
GET /orgs/{org}/blocks/{username}
PUT /orgs/{org}/blocks/{username}
DELETE /orgs/{org}/blocks/{username}
GET /orgs/{org}/failed_invitations
GET /orgs/{org}/hooks
POST /orgs/{org}/hooks
GET /orgs/{org}/hooks/{hook_id}
PATCH /orgs/{org}/hooks/{hook_id}
DELETE /orgs/{org}/hooks/{hook_id}
GET /orgs/{org}/hooks/{hook_id}/config
PATCH /orgs/{org}/hooks/{hook_id}/config
GET /orgs/{org}/hooks/{hook_id}/deliveries
GET /orgs/{org}/hooks/{hook_id}/deliveries/{delivery_id}
POST /orgs/{org}/hooks/{hook_id}/deliveries/{delivery_id}/attempts
POST /orgs/{org}/hooks/{hook_id}/pings
GET /orgs/{org}/installations
GET /orgs/{org}/invitations
POST /orgs/{org}/invitations
DELETE /orgs/{org}/invitations/{invitation_id}
GET /orgs/{org}/invitations/{invitation_id}/teams
GET /orgs/{org}/members
GET /orgs/{org}/members/{username}
DELETE /orgs/{org}/members/{username}
GET /orgs/{org}/memberships/{username}
PUT /orgs/{org}/memberships/{username}
DELETE /orgs/{org}/memberships/{username}
GET /orgs/{org}/outside_collaborators
PUT /orgs/{org}/outside_collaborators/{username}
DELETE /orgs/{org}/outside_collaborators/{username}
GET /orgs/{org}/public_members
GET /orgs/{org}/public_members/{username}
PUT /orgs/{org}/public_members/{username}
DELETE /orgs/{org}/public_members/{username}
GET /user/memberships/orgs/{org}
PATCH /user/memberships/orgs/{org}
GET /user/orgs
```

---

## Packages

```
GET /orgs/{org}/packages
GET /orgs/{org}/packages/{package_type}/{package_name}
DELETE /orgs/{org}/packages/{package_type}/{package_name}
GET /orgs/{org}/packages/{package_type}/{package_name}/versions
GET /orgs/{org}/packages/{package_type}/{package_name}/versions/{package_version_id}
GET /user/packages
GET /user/packages/{package_type}/{package_name}
DELETE /user/packages/{package_type}/{package_name}
GET /user/packages/{package_type}/{package_name}/versions
GET /user/packages/{package_type}/{package_name}/versions/{package_version_id}
```

---

## Pages

```
GET /repos/{owner}/{repo}/pages
POST /repos/{owner}/{repo}/pages
PUT /repos/{owner}/{repo}/pages
DELETE /repos/{owner}/{repo}/pages
GET /repos/{owner}/{repo}/pages/builds
POST /repos/{owner}/{repo}/pages/builds
GET /repos/{owner}/{repo}/pages/builds/latest
GET /repos/{owner}/{repo}/pages/builds/{build_id}
POST /repos/{owner}/{repo}/pages/deployment
GET /repos/{owner}/{repo}/pages/deployments/{pages_deployment_id}
POST /repos/{owner}/{repo}/pages/deployments/{pages_deployment_id}/cancel
GET /repos/{owner}/{repo}/pages/health
```

---

## Projects (V2)

```
GET /orgs/{org}/projectsV2
GET /orgs/{org}/projectsV2/{project_number}
POST /orgs/{org}/projectsV2/{project_number}/drafts
GET /orgs/{org}/projectsV2/{project_number}/fields
GET /orgs/{org}/projectsV2/{project_number}/fields/{field_id}
GET /orgs/{org}/projectsV2/{project_number}/items
POST /orgs/{org}/projectsV2/{project_number}/items
GET /orgs/{org}/projectsV2/{project_number}/items/{item_id}
PATCH /orgs/{org}/projectsV2/{project_number}/items/{item_id}
DELETE /orgs/{org}/projectsV2/{project_number}/items/{item_id}
```

---

## Projects (Classic)

```
GET /projects/columns/{column_id}
PATCH /projects/columns/{column_id}
DELETE /projects/columns/{column_id}
POST /projects/columns/{column_id}/moves
GET /projects/{project_id}/collaborators
PUT /projects/{project_id}/collaborators/{username}
DELETE /projects/{project_id}/collaborators/{username}
GET /projects/{project_id}/collaborators/{username}/permission
```

---

## Pull Requests

```
GET /repos/{owner}/{repo}/pulls
POST /repos/{owner}/{repo}/pulls
GET /repos/{owner}/{repo}/pulls/comments
GET /repos/{owner}/{repo}/pulls/comments/{comment_id}
PATCH /repos/{owner}/{repo}/pulls/comments/{comment_id}
DELETE /repos/{owner}/{repo}/pulls/comments/{comment_id}
GET /repos/{owner}/{repo}/pulls/{pull_number}
PATCH /repos/{owner}/{repo}/pulls/{pull_number}
GET /repos/{owner}/{repo}/pulls/{pull_number}/comments
POST /repos/{owner}/{repo}/pulls/{pull_number}/comments
POST /repos/{owner}/{repo}/pulls/{pull_number}/comments/{comment_id}/replies
GET /repos/{owner}/{repo}/pulls/{pull_number}/commits
GET /repos/{owner}/{repo}/pulls/{pull_number}/files
GET /repos/{owner}/{repo}/pulls/{pull_number}/merge
PUT /repos/{owner}/{repo}/pulls/{pull_number}/merge
GET /repos/{owner}/{repo}/pulls/{pull_number}/requested_reviewers
POST /repos/{owner}/{repo}/pulls/{pull_number}/requested_reviewers
DELETE /repos/{owner}/{repo}/pulls/{pull_number}/requested_reviewers
GET /repos/{owner}/{repo}/pulls/{pull_number}/reviews
POST /repos/{owner}/{repo}/pulls/{pull_number}/reviews
GET /repos/{owner}/{repo}/pulls/{pull_number}/reviews/{review_id}
PUT /repos/{owner}/{repo}/pulls/{pull_number}/reviews/{review_id}
DELETE /repos/{owner}/{repo}/pulls/{pull_number}/reviews/{review_id}
GET /repos/{owner}/{repo}/pulls/{pull_number}/reviews/{review_id}/comments
PUT /repos/{owner}/{repo}/pulls/{pull_number}/reviews/{review_id}/dismissals
POST /repos/{owner}/{repo}/pulls/{pull_number}/reviews/{review_id}/events
PUT /repos/{owner}/{repo}/pulls/{pull_number}/update-branch
```

---

## Rate Limit

```
GET /rate_limit
```

---

## Reactions

```
GET /repos/{owner}/{repo}/comments/{comment_id}/reactions
POST /repos/{owner}/{repo}/comments/{comment_id}/reactions
DELETE /repos/{owner}/{repo}/comments/{comment_id}/reactions/{reaction_id}
GET /repos/{owner}/{repo}/issues/comments/{comment_id}/reactions
POST /repos/{owner}/{repo}/issues/comments/{comment_id}/reactions
DELETE /repos/{owner}/{repo}/issues/comments/{comment_id}/reactions/{reaction_id}
GET /repos/{owner}/{repo}/issues/{issue_number}/reactions
POST /repos/{owner}/{repo}/issues/{issue_number}/reactions
DELETE /repos/{owner}/{repo}/issues/{issue_number}/reactions/{reaction_id}
GET /repos/{owner}/{repo}/pulls/comments/{comment_id}/reactions
POST /repos/{owner}/{repo}/pulls/comments/{comment_id}/reactions
DELETE /repos/{owner}/{repo}/pulls/comments/{comment_id}/reactions/{reaction_id}
GET /repos/{owner}/{repo}/releases/{release_id}/reactions
POST /repos/{owner}/{repo}/releases/{release_id}/reactions
DELETE /repos/{owner}/{repo}/releases/{release_id}/reactions/{reaction_id}
```

---

## Releases

```
GET /repos/{owner}/{repo}/releases
POST /repos/{owner}/{repo}/releases
GET /repos/{owner}/{repo}/releases/assets/{asset_id}
PATCH /repos/{owner}/{repo}/releases/assets/{asset_id}
DELETE /repos/{owner}/{repo}/releases/assets/{asset_id}
POST /repos/{owner}/{repo}/releases/generate-notes
GET /repos/{owner}/{repo}/releases/latest
GET /repos/{owner}/{repo}/releases/tags/{tag}
GET /repos/{owner}/{repo}/releases/{release_id}
PATCH /repos/{owner}/{repo}/releases/{release_id}
DELETE /repos/{owner}/{repo}/releases/{release_id}
GET /repos/{owner}/{repo}/releases/{release_id}/assets
```

---

## Repositories

```
GET /orgs/{org}/repos
POST /orgs/{org}/repos
GET /repos/{owner}/{repo}
PATCH /repos/{owner}/{repo}
DELETE /repos/{owner}/{repo}
GET /repos/{owner}/{repo}/activity
POST /repos/{owner}/{repo}/attestations
GET /repos/{owner}/{repo}/attestations/{subject_digest}
GET /repos/{owner}/{repo}/autolinks
POST /repos/{owner}/{repo}/autolinks
GET /repos/{owner}/{repo}/autolinks/{autolink_id}
DELETE /repos/{owner}/{repo}/autolinks/{autolink_id}
GET /repos/{owner}/{repo}/automated-security-fixes
PUT /repos/{owner}/{repo}/automated-security-fixes
DELETE /repos/{owner}/{repo}/automated-security-fixes
GET /repos/{owner}/{repo}/codeowners/errors
GET /repos/{owner}/{repo}/contents/{path}
PUT /repos/{owner}/{repo}/contents/{path}
DELETE /repos/{owner}/{repo}/contents/{path}
GET /repos/{owner}/{repo}/contributors
POST /repos/{owner}/{repo}/dispatches
GET /repos/{owner}/{repo}/forks
POST /repos/{owner}/{repo}/forks
GET /repos/{owner}/{repo}/hooks
POST /repos/{owner}/{repo}/hooks
GET /repos/{owner}/{repo}/hooks/{hook_id}
PATCH /repos/{owner}/{repo}/hooks/{hook_id}
DELETE /repos/{owner}/{repo}/hooks/{hook_id}
GET /repos/{owner}/{repo}/hooks/{hook_id}/config
PATCH /repos/{owner}/{repo}/hooks/{hook_id}/config
GET /repos/{owner}/{repo}/hooks/{hook_id}/deliveries
GET /repos/{owner}/{repo}/hooks/{hook_id}/deliveries/{delivery_id}
POST /repos/{owner}/{repo}/hooks/{hook_id}/deliveries/{delivery_id}/attempts
POST /repos/{owner}/{repo}/hooks/{hook_id}/pings
POST /repos/{owner}/{repo}/hooks/{hook_id}/tests
GET /repos/{owner}/{repo}/languages
GET /repos/{owner}/{repo}/properties/values
PATCH /repos/{owner}/{repo}/properties/values
GET /repos/{owner}/{repo}/readme
GET /repos/{owner}/{repo}/readme/{dir}
GET /repos/{owner}/{repo}/rules/branches/{branch}
GET /repos/{owner}/{repo}/rulesets
POST /repos/{owner}/{repo}/rulesets
GET /repos/{owner}/{repo}/rulesets/rule-suites
GET /repos/{owner}/{repo}/rulesets/rule-suites/{rule_suite_id}
GET /repos/{owner}/{repo}/rulesets/{ruleset_id}
PUT /repos/{owner}/{repo}/rulesets/{ruleset_id}
DELETE /repos/{owner}/{repo}/rulesets/{ruleset_id}
GET /repos/{owner}/{repo}/tags
GET /repos/{owner}/{repo}/tags/protection
POST /repos/{owner}/{repo}/tags/protection
DELETE /repos/{owner}/{repo}/tags/protection/{tag_protection_id}
GET /repos/{owner}/{repo}/tarball/{ref}
GET /repos/{owner}/{repo}/teams
GET /repos/{owner}/{repo}/topics
PUT /repos/{owner}/{repo}/topics
POST /repos/{owner}/{repo}/transfer
GET /repos/{owner}/{repo}/zipball/{ref}
POST /repos/{template_owner}/{template_repo}/generate
GET /repositories
GET /user/repos
POST /user/repos
GET /users/{username}/repos
```

---

## Search

```
GET /search/code
GET /search/commits
GET /search/issues
GET /search/labels
GET /search/repositories
GET /search/topics
GET /search/users
```

---

## Secret Scanning

```
GET /orgs/{org}/secret-scanning/alerts
GET /repos/{owner}/{repo}/secret-scanning/alerts
GET /repos/{owner}/{repo}/secret-scanning/alerts/{alert_number}
PATCH /repos/{owner}/{repo}/secret-scanning/alerts/{alert_number}
GET /repos/{owner}/{repo}/secret-scanning/alerts/{alert_number}/locations
POST /repos/{owner}/{repo}/secret-scanning/push-protection-bypasses
GET /repos/{owner}/{repo}/secret-scanning/scan-history
```

---

## Security Advisories

```
GET /orgs/{org}/security-advisories
GET /repos/{owner}/{repo}/security-advisories
POST /repos/{owner}/{repo}/security-advisories
POST /repos/{owner}/{repo}/security-advisories/reports
GET /repos/{owner}/{repo}/security-advisories/{ghsa_id}
PATCH /repos/{owner}/{repo}/security-advisories/{ghsa_id}
POST /repos/{owner}/{repo}/security-advisories/{ghsa_id}/cve
POST /repos/{owner}/{repo}/security-advisories/{ghsa_id}/forks
```

---

## Teams

```
GET /orgs/{org}/teams
POST /orgs/{org}/teams
GET /orgs/{org}/teams/{team_slug}
PATCH /orgs/{org}/teams/{team_slug}
DELETE /orgs/{org}/teams/{team_slug}
GET /orgs/{org}/teams/{team_slug}/discussions
POST /orgs/{org}/teams/{team_slug}/discussions
GET /orgs/{org}/teams/{team_slug}/discussions/{discussion_number}
PATCH /orgs/{org}/teams/{team_slug}/discussions/{discussion_number}
DELETE /orgs/{org}/teams/{team_slug}/discussions/{discussion_number}
GET /orgs/{org}/teams/{team_slug}/discussions/{discussion_number}/comments
POST /orgs/{org}/teams/{team_slug}/discussions/{discussion_number}/comments
GET /orgs/{org}/teams/{team_slug}/discussions/{discussion_number}/comments/{comment_number}
PATCH /orgs/{org}/teams/{team_slug}/discussions/{discussion_number}/comments/{comment_number}
DELETE /orgs/{org}/teams/{team_slug}/discussions/{discussion_number}/comments/{comment_number}
GET /orgs/{org}/teams/{team_slug}/invitations
GET /orgs/{org}/teams/{team_slug}/members
GET /orgs/{org}/teams/{team_slug}/memberships/{username}
PUT /orgs/{org}/teams/{team_slug}/memberships/{username}
DELETE /orgs/{org}/teams/{team_slug}/memberships/{username}
GET /orgs/{org}/teams/{team_slug}/projects
GET /orgs/{org}/teams/{team_slug}/projects/{project_id}
PUT /orgs/{org}/teams/{team_slug}/projects/{project_id}
DELETE /orgs/{org}/teams/{team_slug}/projects/{project_id}
GET /orgs/{org}/teams/{team_slug}/repos
GET /orgs/{org}/teams/{team_slug}/repos/{owner}/{repo}
PUT /orgs/{org}/teams/{team_slug}/repos/{owner}/{repo}
DELETE /orgs/{org}/teams/{team_slug}/repos/{owner}/{repo}
GET /orgs/{org}/teams/{team_slug}/teams
GET /teams/{team_id}
PATCH /teams/{team_id}
DELETE /teams/{team_id}
```

---

## Users

```
PATCH /user
GET /user/blocks
GET /user/blocks/{username}
PUT /user/blocks/{username}
DELETE /user/blocks/{username}
PATCH /user/email/visibility
GET /user/emails
POST /user/emails
DELETE /user/emails
GET /user/followers
GET /user/following
GET /user/following/{username}
PUT /user/following/{username}
DELETE /user/following/{username}
GET /user/gpg_keys
POST /user/gpg_keys
GET /user/gpg_keys/{gpg_key_id}
DELETE /user/gpg_keys/{gpg_key_id}
GET /user/keys
POST /user/keys
GET /user/keys/{key_id}
DELETE /user/keys/{key_id}
GET /user/public_emails
POST /user/social_accounts
DELETE /user/social_accounts
GET /user/ssh_signing_keys
POST /user/ssh_signing_keys
GET /user/ssh_signing_keys/{ssh_signing_key_id}
DELETE /user/ssh_signing_keys/{ssh_signing_key_id}
GET /users
GET /users/{username}
GET /users/{username}/followers
GET /users/{username}/following
GET /users/{username}/following/{target_user}
GET /users/{username}/gpg_keys
GET /users/{username}/keys
GET /users/{username}/social_accounts
GET /users/{username}/ssh_signing_keys
```

---

## Reference

- [Official Documentation](https://docs.github.com/en/rest)
- API Version: 2022-11-28 (latest)

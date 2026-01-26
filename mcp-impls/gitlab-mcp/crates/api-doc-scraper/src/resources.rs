use std::{collections::HashMap, path::PathBuf};

/// Category of API resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceCategory {
    Project,
    Group,
    Standalone,
    Templates,
}

impl ResourceCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Group => "group",
            Self::Standalone => "standalone",
            Self::Templates => "templates",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Project => "Project Resources",
            Self::Group => "Group Resources",
            Self::Standalone => "Standalone Resources",
            Self::Templates => "Template Resources",
        }
    }
}

/// Represents an API documentation resource
#[derive(Debug, Clone)]
pub struct ApiResource {
    pub name: String,
    pub url_slug: String,
    pub category: ResourceCategory,
    pub output_path: PathBuf,
}

impl ApiResource {
    pub fn new(name: impl Into<String>, url_slug: impl Into<String>, category: ResourceCategory) -> Self {
        let name = name.into();
        let url_slug = url_slug.into();
        let output_path = Self::build_output_path(&name, category);
        Self {
            name,
            url_slug,
            category,
            output_path,
        }
    }

    fn build_output_path(name: &str, category: ResourceCategory) -> PathBuf {
        let filename = name.to_lowercase().replace(' ', "_");
        PathBuf::from(category.as_str()).join(format!("{}.md", filename))
    }

    pub fn url(&self) -> String {
        format!("ee/api/{}.html", self.url_slug)
    }
}

/// Get all GitLab API resources to scrape
pub fn get_all_resources() -> Vec<ApiResource> {
    let mut resources = Vec::new();

    // Project Resources
    let project_resources = [
        ("Access Requests", "access_requests"),
        ("Access Tokens", "access_tokens"),
        ("Agents", "cluster_agents"),
        ("Application Settings", "application_settings"),
        ("Approvals", "merge_request_approvals"),
        ("Audit Events", "audit_events"),
        ("Award Emoji", "award_emoji"),
        ("Badges", "badges"),
        ("Board", "boards"),
        ("Branches", "branches"),
        ("Cherry Pick Commits", "cherry_pick_commits"),
        ("Clusters", "clusters"),
        ("Commits", "commits"),
        ("Composer Packages", "composer"),
        ("Container Repository Protection Rules", "container_repository_protection_rules"),
        ("Container Registry", "container_registry"),
        ("Custom Attributes", "custom_attributes"),
        ("Cycle Analytics", "cycle_analytics"),
        ("Deploy Keys", "deploy_keys"),
        ("Deploy Tokens", "deploy_tokens"),
        ("Deployments", "deployments"),
        ("Discussions", "discussions"),
        ("Draft Notes", "draft_notes"),
        ("Environments", "environments"),
        ("Epics", "epics"),
        ("Error Tracking", "error_tracking"),
        ("Events", "events"),
        ("External Status Checks", "external_status_checks"),
        ("Feature Flags", "feature_flags"),
        ("Feature Flags User Lists", "feature_flags_user_lists"),
        ("Freeze Periods", "freeze_periods"),
        ("Go Packages", "go"),
        ("GitLab CI YAML Templates", "gitlab_ci_yml_templates"),
        ("Group Import", "group_import"),
        ("Helm Packages", "helm_repository"),
        ("Import GitHub", "import_github"),
        ("Integrations", "integrations"),
        ("Invitations", "invitations"),
        ("Issue Boards", "boards"),
        ("Issue Links", "issue_links"),
        ("Issues", "issues"),
        ("Issues Statistics", "issues_statistics"),
        ("Iterations", "iterations"),
        ("Job Artifacts", "job_artifacts"),
        ("Jobs", "jobs"),
        ("Job Token Scope", "job_token_scope"),
        ("Keys", "keys"),
        ("Labels", "labels"),
        ("License Templates", "license_templates"),
        ("Lint", "lint"),
        ("Markdown", "markdown"),
        ("Maven Packages", "maven"),
        ("Members", "members"),
        ("Merge Request Approvals", "merge_request_approvals"),
        ("Merge Requests", "merge_requests"),
        ("Merge Trains", "merge_trains"),
        ("Metadata", "metadata"),
        ("Milestones", "milestones"),
        ("Model Registry", "ml_models"),
        ("Namespaces", "namespaces"),
        ("Notes", "notes"),
        ("Notification Settings", "notification_settings"),
        ("NPM Packages", "npm"),
        ("NuGet Packages", "nuget"),
        ("Packages", "packages"),
        ("Pages Domains", "pages_domains"),
        ("Pages", "pages"),
        ("Pipeline Schedules", "pipeline_schedules"),
        ("Pipeline Triggers", "pipeline_triggers"),
        ("Pipelines", "pipelines"),
        ("Project Access Tokens", "project_access_tokens"),
        ("Project Import", "project_import"),
        ("Project Snippets", "snippets"),
        ("Project Templates", "project_templates"),
        ("Projects", "projects"),
        ("Protected Branches", "protected_branches"),
        ("Protected Environments", "protected_environments"),
        ("Protected Packages", "protected_packages"),
        ("Protected Tags", "protected_tags"),
        ("PyPI Packages", "pypi"),
        ("Push Rules", "push_rules"),
        ("Python Packages", "pypi"),
        ("Releases", "releases"),
        ("Release Links", "release_links"),
        ("Remote Mirrors", "remote_mirrors"),
        ("Repositories", "repositories"),
        ("Repository Files", "repository_files"),
        ("Repository Submodules", "repository_submodules"),
        ("Resource Label Events", "resource_label_events"),
        ("Resource Milestone Events", "resource_milestone_events"),
        ("Resource State Events", "resource_state_events"),
        ("Resource Weight Events", "resource_weight_events"),
        ("Revert Commits", "revert_commits"),
        ("Ruby Gems Packages", "rubygems"),
        ("Runners", "runners"),
        ("Search", "search"),
        ("Secure Files", "secure_files"),
        ("Service Accounts", "service_accounts"),
        ("Settings", "settings"),
        ("Snippets", "snippets"),
        ("Suggestions", "suggestions"),
        ("Tags", "tags"),
        ("Terraform Modules", "terraform_modules"),
        ("Todos", "todos"),
        ("Topics", "topics"),
        ("Users", "users"),
        ("Validate CI YAML", "ci_lint"),
        ("Vulnerability Findings", "vulnerability_findings"),
        ("Vulnerability Exports", "vulnerability_exports"),
        ("Vulnerabilities", "vulnerabilities"),
        ("Web commits", "web_commits"),
        ("Wikis", "wikis"),
        ("Variables", "ci_variables"),
    ];

    for (name, slug) in project_resources {
        resources.push(ApiResource::new(name, slug, ResourceCategory::Project));
    }

    // Group Resources
    let group_resources = [
        ("Access Requests", "access_requests"),
        ("Access Tokens", "group_access_tokens"),
        ("Badges", "group_badges"),
        ("Boards", "group_boards"),
        ("Custom Attributes", "custom_attributes"),
        ("Debian Packages", "debian_group_packages"),
        ("Deploy Tokens", "deploy_tokens"),
        ("Discussion", "group_discussions"),
        ("Epics", "epics"),
        ("Epic Issues", "epic_issues"),
        ("Epic Links", "epic_links"),
        ("Groups", "groups"),
        ("Invitations", "invitations"),
        ("Issues", "issues"),
        ("Issues Statistics", "issues_statistics"),
        ("Iterations", "iterations"),
        ("Labels", "group_labels"),
        ("LDAP Group Links", "ldap_group_links"),
        ("Member Roles", "member_roles"),
        ("Members", "members"),
        ("Merge Requests", "merge_requests"),
        ("Milestones", "milestones"),
        ("Notification Settings", "notification_settings"),
        ("Packages", "group_packages"),
        ("Projects", "group_projects"),
        ("Related Epics", "related_epics"),
        ("Releases", "group_releases"),
        ("Resource Label Events", "resource_label_events"),
        ("Runners", "group_runners"),
        ("SAML Group Links", "saml_group_links"),
        ("SCIM", "scim"),
        ("Search", "group_search"),
        ("SSH Certificates", "group_ssh_certificates"),
        ("Variables", "group_variables"),
        ("Wikis", "group_wikis"),
    ];

    for (name, slug) in group_resources {
        resources.push(ApiResource::new(name, slug, ResourceCategory::Group));
    }

    // Standalone Resources
    let standalone_resources = [
        ("Appearance", "appearance"),
        ("Applications", "applications"),
        ("Audit Events", "audit_events"),
        ("Avatar", "avatar"),
        ("Broadcast Messages", "broadcast_messages"),
        ("Code Snippets", "snippets"),
        ("Code Suggestions", "code_suggestions"),
        ("Custom Attributes", "custom_attributes"),
        ("Deploy Keys", "deploy_keys"),
        ("Deploy Tokens", "deploy_tokens"),
        ("Dependency List Exports", "dependency_list_exports"),
        ("Dockerfile Templates", "dockerfile_templates"),
        ("Events", "events"),
        ("Features", "features"),
        ("Geo Nodes", "geo_nodes"),
        ("Gitignore Templates", "gitignore_templates"),
        ("Group Activity Analytics", "group_activity_analytics"),
        ("Group Repository Storage Moves", "group_repository_storage_moves"),
        ("Import Bitbucket Server", "import_bitbucket_server"),
        ("Import GitHub", "import_github"),
        ("Instance Clusters", "instance_clusters"),
        ("Instance Variables", "instance_variables"),
        ("Issues", "issues"),
        ("Issues Statistics", "issues_statistics"),
        ("Jobs", "jobs"),
        ("Keys", "keys"),
        ("License", "license"),
        ("License Templates", "license_templates"),
        ("Markdown", "markdown"),
        ("Merge Requests", "merge_requests"),
        ("Namespaces", "namespaces"),
        ("Notification Settings", "notification_settings"),
        ("Pages Domains", "pages_domains"),
        ("Personal Access Tokens", "personal_access_tokens"),
        ("Plan Limits", "plan_limits"),
        ("Project Repository Storage Moves", "project_repository_storage_moves"),
        ("Projects", "projects"),
        ("Protected Branches", "protected_branches"),
        ("Runners", "runners"),
        ("Search", "search"),
        ("Service Data", "service_data"),
        ("Settings", "settings"),
        ("Sidekiq Metrics", "sidekiq_metrics"),
        ("Sidekiq Queues", "sidekiq_queues"),
        ("Snippet Repository Storage Moves", "snippet_repository_storage_moves"),
        ("Statistics", "statistics"),
        ("Suggestions", "suggestions"),
        ("System Hooks", "hooks"),
        ("Todos", "todos"),
        ("Token Information", "token_information"),
        ("Topics", "topics"),
        ("Users", "users"),
        ("Version", "version"),
        ("Web commits", "web_commits"),
    ];

    for (name, slug) in standalone_resources {
        resources.push(ApiResource::new(name, slug, ResourceCategory::Standalone));
    }

    // Template Resources
    let template_resources = [
        ("Dockerfiles", "dockerfile_templates"),
        (".gitignore", "gitignore_templates"),
        ("GitLab CI/CD", "gitlab_ci_yml_templates"),
        ("Licenses", "license_templates"),
    ];

    for (name, slug) in template_resources {
        resources.push(ApiResource::new(name, slug, ResourceCategory::Templates));
    }

    resources
}

/// Get resources grouped by category
pub fn get_resources_by_category() -> HashMap<ResourceCategory, Vec<ApiResource>> {
    let mut grouped: HashMap<ResourceCategory, Vec<ApiResource>> = HashMap::new();

    for resource in get_all_resources() {
        grouped.entry(resource.category).or_default().push(resource);
    }

    grouped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_resources() {
        let resources = get_all_resources();
        assert!(!resources.is_empty());
        println!("Total resources: {}", resources.len());
    }

    #[test]
    fn test_resource_url() {
        let resource = ApiResource::new("Issues", "issues", ResourceCategory::Project);
        assert_eq!(resource.url(), "ee/api/issues.html");
    }

    #[test]
    fn test_output_path() {
        let resource = ApiResource::new("Issues", "issues", ResourceCategory::Project);
        assert_eq!(resource.output_path, PathBuf::from("project/issues.md"));
    }
}

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gitlab-mcp")]
#[command(about = "GitLab MCP Client - CLI tool for GitLab operations", long_about = None)]
#[command(version)]
pub struct Cli {
    /// GitLab instance URL
    #[arg(long)]
    pub url: Option<String>,

    /// GitLab personal access token
    #[arg(long)]
    pub token: Option<String>,

    /// Output format (table, json, plain)
    #[arg(long, default_value = "table")]
    pub output: String,

    /// Disable colors
    #[arg(long, action = clap::ArgAction::SetFalse)]
    pub color: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Configuration management
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Project operations
    #[command(subcommand)]
    Project(ProjectCommands),

    /// Issue operations
    #[command(subcommand)]
    Issue(IssueCommands),

    /// Merge Request operations
    #[command(subcommand)]
    Mr(MrCommands),

    /// Pipeline operations
    #[command(subcommand)]
    Pipeline(PipelineCommands),

    /// Repository file operations
    #[command(subcommand)]
    Repo(RepoCommands),

    /// Wiki operations
    #[command(subcommand)]
    Wiki(WikiCommands),

    /// Branch operations
    #[command(subcommand)]
    Branch(BranchCommands),

    /// Commit operations
    #[command(subcommand)]
    Commit(CommitCommands),

    /// Tag operations
    #[command(subcommand)]
    Tag(TagCommands),

    /// Milestone operations
    #[command(subcommand)]
    Milestone(MilestoneCommands),

    /// Environment operations
    #[command(subcommand)]
    Environment(EnvironmentCommands),

    /// Release operations
    #[command(subcommand)]
    Release(ReleaseCommands),

    /// User operations
    #[command(subcommand)]
    User(UserCommands),
}

/// Project commands
#[derive(Subcommand)]
pub enum ProjectCommands {
    /// List projects
    List {
        /// Search string
        #[arg(short, long)]
        search: Option<String>,
        /// Number of items per page
        #[arg(long, default_value = "20")]
        per_page: usize,
        /// Page number
        #[arg(long)]
        page: Option<usize>,
    },
    /// Get project details
    Get {
        /// Project ID or path
        project_id: String,
    },
    /// Get project members
    Members {
        /// Project ID or path
        project_id: String,
    },
}

/// Issue commands
#[derive(Subcommand)]
pub enum IssueCommands {
    /// List issues
    List {
        /// Project ID or path
        project_id: String,
        /// Issue state (opened, closed, all)
        #[arg(long)]
        state: Option<String>,
        /// Labels filter
        #[arg(long)]
        labels: Option<String>,
        /// Assignee filter
        #[arg(long)]
        assignee: Option<String>,
    },
    /// Get issue details
    Get {
        /// Project ID or path
        project_id: String,
        /// Issue IID
        issue_iid: u64,
    },
    /// Create an issue
    Create {
        /// Project ID or path
        project_id: String,
        /// Issue title
        #[arg(long)]
        title: String,
        /// Issue description
        #[arg(long)]
        description: Option<String>,
        /// Issue labels (comma-separated)
        #[arg(long)]
        labels: Option<String>,
        /// Assignee username(s)
        #[arg(long)]
        assignee: Option<String>,
    },
    /// Add note to issue
    Note {
        /// Project ID or path
        project_id: String,
        /// Issue IID
        issue_iid: u64,
        /// Note body
        #[arg(long)]
        body: String,
    },
}

/// Merge Request commands
#[derive(Subcommand)]
pub enum MrCommands {
    /// List merge requests
    List {
        /// Project ID or path
        project_id: String,
        /// MR state (opened, closed, merged, all)
        #[arg(long)]
        state: Option<String>,
    },
    /// Get MR details
    Get {
        /// Project ID or path
        project_id: String,
        /// MR IID
        mr_iid: u64,
    },
    /// Create a merge request
    Create {
        /// Project ID or path
        project_id: String,
        /// Source branch
        #[arg(long)]
        source_branch: String,
        /// Target branch
        #[arg(long)]
        target_branch: String,
        /// MR title
        #[arg(long)]
        title: String,
    },
    /// Merge a merge request
    Merge {
        /// Project ID or path
        project_id: String,
        /// MR IID
        mr_iid: u64,
        /// Merge message
        #[arg(long)]
        message: Option<String>,
    },
}

/// Pipeline commands
#[derive(Subcommand)]
pub enum PipelineCommands {
    /// List pipelines
    List {
        /// Project ID or path
        project_id: String,
        /// Ref filter
        #[arg(long)]
        r#ref: Option<String>,
    },
    /// Get pipeline details
    Get {
        /// Project ID or path
        project_id: String,
        /// Pipeline ID
        pipeline_id: u64,
    },
    /// Get job log
    JobLog {
        /// Project ID or path
        project_id: String,
        /// Job ID
        job_id: u64,
    },
}

/// Repository commands
#[derive(Subcommand)]
pub enum RepoCommands {
    /// List files
    List {
        /// Project ID or path
        project_id: String,
        /// Path in repository
        #[arg(long)]
        path: Option<String>,
        /// Ref (branch, tag, commit)
        #[arg(long)]
        r#ref: Option<String>,
    },
    /// Get file content
    Get {
        /// Project ID or path
        project_id: String,
        /// File path
        #[arg(long)]
        file_path: String,
        /// Ref (branch, tag, commit)
        #[arg(long)]
        r#ref: Option<String>,
    },
}

/// Wiki commands
#[derive(Subcommand)]
pub enum WikiCommands {
    /// List wiki pages
    List {
        /// Project ID or path
        project_id: String,
    },
    /// Get wiki page
    Get {
        /// Project ID or path
        project_id: String,
        /// Wiki page slug
        slug: String,
    },
}

/// Branch commands
#[derive(Subcommand)]
pub enum BranchCommands {
    /// List branches
    List {
        /// Project ID or path
        project_id: String,
        /// Search string
        #[arg(long)]
        search: Option<String>,
    },
    /// Create a branch
    Create {
        /// Project ID or path
        project_id: String,
        /// Branch name
        #[arg(long)]
        name: String,
        /// Source ref
        #[arg(long)]
        from: String,
    },
    /// Delete a branch
    Delete {
        /// Project ID or path
        project_id: String,
        /// Branch name
        #[arg(long)]
        name: String,
    },
}

/// Commit commands
#[derive(Subcommand)]
pub enum CommitCommands {
    /// List commits
    List {
        /// Project ID or path
        project_id: String,
        /// Ref name
        #[arg(long)]
        ref_name: Option<String>,
    },
    /// Get commit details
    Get {
        /// Project ID or path
        project_id: String,
        /// Commit SHA
        sha: String,
    },
}

/// Tag commands
#[derive(Subcommand)]
pub enum TagCommands {
    /// List tags
    List {
        /// Project ID or path
        project_id: String,
    },
    /// Create a tag
    Create {
        /// Project ID or path
        project_id: String,
        /// Tag name
        #[arg(long)]
        name: String,
        /// Target ref
        #[arg(long)]
        r#ref: String,
        /// Message
        #[arg(long)]
        message: Option<String>,
    },
}

/// Milestone commands
#[derive(Subcommand)]
pub enum MilestoneCommands {
    /// List milestones
    List {
        /// Project ID or path
        project_id: String,
    },
    /// Get milestone
    Get {
        /// Project ID or path
        project_id: String,
        /// Milestone ID
        milestone_id: u64,
    },
}

/// Environment commands
#[derive(Subcommand)]
pub enum EnvironmentCommands {
    /// List environments
    List {
        /// Project ID or path
        project_id: String,
    },
    /// Get environment
    Get {
        /// Project ID or path
        project_id: String,
        /// Environment ID
        environment_id: u64,
    },
}

/// Release commands
#[derive(Subcommand)]
pub enum ReleaseCommands {
    /// List releases
    List {
        /// Project ID or path
        project_id: String,
    },
    /// Get release
    Get {
        /// Project ID or path
        project_id: String,
        /// Tag name
        tag_name: String,
    },
}

/// User commands
#[derive(Subcommand)]
pub enum UserCommands {
    /// Get current user
    Me,
    /// List users
    List {
        /// Search string
        #[arg(short, long)]
        search: Option<String>,
    },
}

/// Config commands
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set GitLab URL
    SetUrl {
        /// GitLab instance URL
        url: String,
    },
    /// Set GitLab token
    SetToken {
        /// GitLab personal access token
        token: String,
    },
    /// Set log level
    SetLogLevel {
        /// Log level (trace, debug, info, warn, error)
        level: String,
    },
    /// Show config file location
    Path,
}

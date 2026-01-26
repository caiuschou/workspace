use thiserror::Error;

/// GitLab MCP Server errors
#[derive(Error, Debug)]
pub enum GitLabError {
    #[error("GitLab API error: {0}")]
    ApiError(#[from] reqwest::Error),

    #[error("GitLab API returned error: {status} - {message}")]
    ApiResponse { status: u16, message: String },

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl GitLabError {
    pub fn api_response(status: u16, message: impl Into<String>) -> Self {
        Self::ApiResponse {
            status,
            message: message.into(),
        }
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound(resource.into())
    }

    pub fn invalid_parameter(param: impl Into<String>) -> Self {
        Self::InvalidParameter(param.into())
    }

    pub fn auth_error(msg: impl Into<String>) -> Self {
        Self::AuthError(msg.into())
    }

    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }
}

pub type Result<T> = std::result::Result<T, GitLabError>;

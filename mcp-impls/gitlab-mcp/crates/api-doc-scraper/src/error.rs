use thiserror::Error;

/// Documentation scraper errors
#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid HTML: {0}")]
    InvalidHtml(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Max retries exceeded for {0}")]
    MaxRetriesExceeded(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl ScraperError {
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self::ParseError(msg.into())
    }

    pub fn invalid_html(msg: impl Into<String>) -> Self {
        Self::InvalidHtml(msg.into())
    }

    pub fn max_retries_exceeded(url: impl Into<String>) -> Self {
        Self::MaxRetriesExceeded(url.into())
    }

    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }
}

pub type Result<T> = std::result::Result<T, ScraperError>;

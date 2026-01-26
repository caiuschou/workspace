pub mod client;
pub mod error;
pub mod parser;
pub mod resources;

pub use client::DocScraperClient;
pub use error::{Result, ScraperError};
pub use parser::HtmlParser;
pub use resources::{ApiResource, ResourceCategory, get_all_resources, get_resources_by_category};

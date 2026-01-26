use scraper::{Html, Selector};
use tracing::debug;

use crate::error::{Result, ScraperError};

/// HTML parser for extracting documentation content
pub struct HtmlParser;

impl HtmlParser {
    /// Extract the main documentation content from a GitLab docs page
    pub fn extract_main_content(html: &str) -> Result<String> {
        let document = Html::parse_document(html);

        // GitLab docs use a .content class for main content
        let content_selector = Selector::parse(".content").unwrap();
        let article_selector = Selector::parse("article").unwrap();
        let main_selector = Selector::parse("main").unwrap();

        // Try to find content in order of preference
        let content = document
            .select(&content_selector)
            .next()
            .or_else(|| document.select(&article_selector).next())
            .or_else(|| document.select(&main_selector).next())
            .ok_or_else(|| ScraperError::parse_error("Could not find main content"))?;

        Ok(content.html())
    }

    /// Convert HTML to Markdown
    pub fn html_to_markdown(html: &str) -> Result<String> {
        // Use html2md library for conversion
        let markdown = html2md::parse_html(html);
        Ok(markdown)
    }

    /// Extract title from HTML
    pub fn extract_title(html: &str) -> Option<String> {
        let document = Html::parse_document(html);

        // Try h1 first
        let h1_selector = Selector::parse("h1").unwrap();
        if let Some(h1) = document.select(&h1_selector).next() {
            let text = h1.text().collect::<String>();
            if !text.trim().is_empty() {
                return Some(text.trim().to_string());
            }
        }

        // Try title tag
        let title_selector = Selector::parse("title").unwrap();
        if let Some(title) = document.select(&title_selector).next() {
            let text = title.text().collect::<String>();
            let text = text.trim().trim_end_matches(" | GitLab").trim();
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }

        None
    }

    /// Clean and convert HTML to final markdown
    pub fn process_page(html: &str) -> Result<(String, String)> {
        debug!("Processing HTML page");

        // Extract title
        let title = Self::extract_title(html).unwrap_or_else(|| "Untitled".to_string());

        // Extract main content
        let content_html = Self::extract_main_content(html)?;

        // Convert to markdown
        let markdown = Self::html_to_markdown(&content_html)?;

        // Clean up markdown
        let markdown = Self::clean_markdown(&markdown);

        Ok((title, markdown))
    }

    /// Clean up markdown content
    fn clean_markdown(markdown: &str) -> String {
        let mut result = markdown.to_string();

        // Remove excessive blank lines
        result = result.split('\n')
            .filter(|line| !line.trim().is_empty() || {
                // Keep some blank lines but not more than 2 in a row
                true
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Collapse multiple blank lines
        while result.contains("\n\n\n\n") {
            result = result.replace("\n\n\n\n", "\n\n\n");
        }
        while result.contains("\n\n\n") {
            result = result.replace("\n\n\n", "\n\n");
        }

        // Trim leading and trailing whitespace
        result = result.trim().to_string();

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title_from_html() {
        let html = r#"<html><head><title>Issues API | GitLab</title></head><body><h1>Issues</h1></body></html>"#;
        let title = HtmlParser::extract_title(html);
        assert_eq!(title, Some("Issues".to_string()));
    }

    #[test]
    fn test_extract_title_from_h1() {
        let html = r#"<html><body><h1>Projects API</h1></body></html>"#;
        let title = HtmlParser::extract_title(html);
        assert_eq!(title, Some("Projects API".to_string()));
    }

    #[test]
    fn test_clean_markdown() {
        let markdown = "Hello\n\n\n\nWorld";
        let cleaned = HtmlParser::clean_markdown(markdown);
        assert_eq!(cleaned, "Hello\n\nWorld");
    }
}

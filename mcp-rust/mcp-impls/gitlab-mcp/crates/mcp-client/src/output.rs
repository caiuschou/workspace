use console::{style, Color};
use serde_json::Value;
use tabled::Tabled;

/// Output formatter
pub struct OutputFormatter {
    format: String,
    color: bool,
}

impl OutputFormatter {
    pub fn new(format: &str, color: bool) -> Self {
        Self {
            format: format.to_string(),
            color,
        }
    }

    /// Print output
    pub fn print(&self, output: String) {
        if self.format == "json" {
            // Try to parse as JSON for pretty printing
            if let Ok(value) = serde_json::from_str::<Value>(&output) {
                println!("{}", serde_json::to_string_pretty(&value).unwrap_or(output));
            } else {
                println!("{}", output);
            }
        } else {
            println!("{}", output);
        }
    }

    /// Print success message
    pub fn success(&self, message: &str) {
        if self.color {
            println!("{} {}", style("✓").fg(Color::Green), message);
        } else {
            println!("✓ {}", message);
        }
    }

    /// Print error message
    pub fn error(&self, message: &str) {
        if self.color {
            eprintln!("{} {}", style("✗").fg(Color::Red), message);
        } else {
            eprintln!("✗ {}", message);
        }
    }

    /// Print info message
    pub fn info(&self, message: &str) {
        if self.color {
            println!("{} {}", style("ℹ").fg(Color::Blue), message);
        } else {
            println!("ℹ {}", message);
        }
    }

    /// Check if output format is table
    pub fn is_table(&self) -> bool {
        self.format == "table"
    }

    /// Get the output format
    pub fn format(&self) -> &str {
        &self.format
    }
}

/// Table row trait
pub trait TableRow: Tabled {
    fn headers() -> Vec<&'static str>;
}

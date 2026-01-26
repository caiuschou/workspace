use mcp_core::types::{BaseMetadata, Icons, Implementation};

pub fn implementation(name: &str) -> Implementation {
    Implementation {
        base: BaseMetadata {
            name: name.to_string(),
            title: None,
        },
        icons: Icons { icons: None },
        version: "0.1.0".to_string(),
        website_url: None,
        description: None,
    }
}

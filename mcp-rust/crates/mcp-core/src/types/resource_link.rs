use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Resource;

/// Resource link content block.
///
/// A resource link is a reference to a resource that can be returned by tools.
/// Unlike embedded resources, resource links do not include the resource content
/// directly - clients must fetch the resource separately if needed.
///
/// Note: Resource links returned by tools are not guaranteed to appear in the
/// results of `resources/list` requests.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ResourceLink {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub resource: Resource,
}

impl ResourceLink {
    /// Create a new resource link from a resource.
    pub fn new(resource: Resource) -> Self {
        Self {
            kind: "resource_link".to_string(),
            resource,
        }
    }

    /// Create a new resource link with the given URI and name.
    pub fn with_uri(uri: impl Into<String>, name: impl Into<String>) -> Self {
        use super::{BaseMetadata, Icons};

        Self {
            kind: "resource_link".to_string(),
            resource: Resource {
                base: BaseMetadata {
                    name: name.into(),
                    title: None,
                },
                icons: Icons::default(),
                uri: uri.into(),
                description: None,
                mime_type: None,
                annotations: None,
                meta: None,
            },
        }
    }

    /// Set the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.resource.description = Some(description.into());
        self
    }

    /// Set the MIME type.
    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.resource.mime_type = Some(mime_type.into());
        self
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.resource.base.title = Some(title.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_link_serialization() {
        let link = ResourceLink::with_uri("file:///example/file.txt", "example-file")
            .description("An example file")
            .mime_type("text/plain");

        let json = serde_json::to_value(&link).unwrap();

        assert_eq!(json["type"], "resource_link");
        assert_eq!(json["uri"], "file:///example/file.txt");
        assert_eq!(json["name"], "example-file");
        assert_eq!(json["description"], "An example file");
        assert_eq!(json["mimeType"], "text/plain");
    }

    #[test]
    fn test_resource_link_deserialization() {
        let json = r#"{
            "type": "resource_link",
            "uri": "https://example.com/resource",
            "name": "test-resource",
            "description": "A test resource",
            "mimeType": "application/json"
        }"#;

        let link: ResourceLink = serde_json::from_str(json).unwrap();

        assert_eq!(link.kind, "resource_link");
        assert_eq!(link.resource.uri, "https://example.com/resource");
        assert_eq!(link.resource.base.name, "test-resource");
        assert_eq!(link.resource.description, Some("A test resource".to_string()));
        assert_eq!(link.resource.mime_type, Some("application/json".to_string()));
    }

    #[test]
    fn test_content_block_with_resource_link() {
        use super::super::ContentBlock;

        let link = ResourceLink::with_uri("file:///test.txt", "test");
        let block = ContentBlock::ResourceLink(link);

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "resource_link");
        assert_eq!(json["uri"], "file:///test.txt");
    }
}

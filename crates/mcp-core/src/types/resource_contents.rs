use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{BlobResourceContents, TextResourceContents};

/// Resource contents returned by resources/read.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum ResourceContents {
    Text(TextResourceContents),
    Blob(BlobResourceContents),
}

//! Role-gen: LLM-based role tree generator using langgraph-rust.
//!
//! Expands a root role (e.g. CEO) into a full hierarchy by repeatedly
//! calling the LLM for each role's description and direct subordinates.

pub mod check;
pub mod config;
pub mod expand;
pub mod output;
pub mod state;

pub use check::run_check;
pub use config::{resolve_prompt_path, Config};
pub use expand::{ExpandNode, LlmRoleOutput, SubordinateSpec};
pub use output::{format_roles_to_text, print_role_tree};
pub use state::{CollaborationItem, QueueItem, Role, RoleGenState, SkillNode};

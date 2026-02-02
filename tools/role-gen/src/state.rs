//! State and role types for the role expansion graph.
//!
//! **Interaction**: Consumed by `expand::ExpandNode` and by `main` when building
//! the initial state and printing the result.

use serde::{Deserialize, Serialize};

/// One node in the skill tree: a skill name and optional children (sub-skills).
/// Used for recursive skill decomposition; leaf nodes (no children) get a dedicated position.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SkillNode {
    /// Skill name.
    #[serde(default)]
    pub name: String,
    /// Child skills (recursive split). Empty = leaf skill → create position for this skill.
    #[serde(default)]
    pub children: Vec<SkillNode>,
}

/// One collaboration item: with which role, and what contents (list).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CollaborationItem {
    /// Collaborating role name.
    #[serde(default)]
    pub role: String,
    /// Collaboration contents: specific items, input/output boundaries, etc.
    #[serde(default)]
    pub contents: Vec<String>,
}

/// A single role in the tree: name, description, background, objectives, skills/skill_tree, collaboration, parent, and subordinate ids.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub background: String,
    #[serde(default)]
    pub objectives: Vec<String>,
    /// Flattened skill list (derived from skill_tree for display). Kept for backward compat.
    #[serde(default)]
    pub skills: Vec<String>,
    /// Skill tree: recursive structure; leaf skills get a dedicated position.
    #[serde(default)]
    pub skill_tree: Vec<SkillNode>,
    /// If Some(s), this role is a position dedicated to skill s (created for a leaf in parent's skill tree).
    #[serde(default)]
    pub is_position_for_skill: Option<String>,
    /// Collaboration with other roles: list of role name and content items.
    #[serde(default)]
    pub collaboration: Vec<CollaborationItem>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub subordinate_ids: Vec<String>,
}

/// One item in the expansion queue: role name to expand and current depth.
#[derive(Clone, Debug)]
pub struct QueueItem {
    pub role_name: String,
    pub parent_id: Option<String>,
    pub depth: u32,
}

/// State for the role expansion graph.
///
/// **Interaction**: Passed into and out of `ExpandNode::run`; initial state
/// has `queue = [root role]`, `roles = []`; final state has `queue = []` and
/// `roles` as the full tree.
#[derive(Clone, Debug, Default)]
pub struct RoleGenState {
    pub roles: Vec<Role>,
    pub queue: Vec<QueueItem>,
    pub depth_limit: u32,
}

impl RoleGenState {
    /// Next id for a new role (simple counter based on current roles len).
    pub fn next_id(&self) -> String {
        format!("role_{}", self.roles.len())
    }
}

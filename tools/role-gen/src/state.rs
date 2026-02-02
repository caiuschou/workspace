//! State and role types for the role expansion graph.
//!
//! **Interaction**: Consumed by `expand::ExpandNode` and by `main` when building
//! the initial state and printing the result.

use serde::{Deserialize, Serialize};

/// One node in the skill tree: a skill name and optional children (sub-skills).
/// Used for recursive skill decomposition; leaf nodes (no children) get a dedicated 岗位.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SkillNode {
    /// Skill name.
    #[serde(default)]
    pub name: String,
    /// Child skills (recursive split). Empty = leaf skill → create 岗位 for this skill.
    #[serde(default)]
    pub children: Vec<SkillNode>,
}

/// One collaboration item: with which role, and what contents (list).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CollaborationItem {
    /// 协同对象角色名称。
    #[serde(default)]
    pub role: String,
    /// 协同内容列表：具体事项、输入输出等。
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
    /// Skill tree: recursive structure; leaf skills get a dedicated 岗位.
    #[serde(default)]
    pub skill_tree: Vec<SkillNode>,
    /// If Some(s), this role is a 岗位 dedicated to skill s (created for a leaf in parent's skill tree).
    #[serde(default)]
    pub is_position_for_skill: Option<String>,
    /// 与其他角色的协同：列表，每项明确角色与内容列表。
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

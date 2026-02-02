//! Expand node: consumes the queue, calls LLM for each role, and appends
//! roles and new queue items. Runs in a single graph step (internal loop).
//!
//! **Interaction**: Implements `langgraph::Node<RoleGenState>`; holds
//! `Box<dyn LlmClient>` (e.g. ChatOpenAI). Used by main when building the graph.

use std::sync::Arc;

use async_trait::async_trait;
use langgraph::{AgentError, Message, Next, Node};
use serde::Deserialize;

use crate::state::{CollaborationItem, QueueItem, Role, RoleGenState, SkillNode};

/// JSON shape we ask the LLM to return for one role expansion.
#[derive(Clone, Debug, Deserialize)]
pub struct LlmRoleOutput {
    pub description: String,
    #[serde(default)]
    pub background: String,
    #[serde(default)]
    pub objectives: Vec<String>,
    /// Flat skills (legacy). If skill_tree is empty, we build tree from this (each skill = leaf).
    #[serde(default)]
    pub skills: Vec<String>,
    /// Skill tree: recursive split; leaf skills get a dedicated 岗位.
    #[serde(default)]
    pub skill_tree: Vec<SkillNode>,
    /// 与其他角色的协同：列表，每项明确角色与内容列表。
    #[serde(default)]
    pub collaboration: Vec<CollaborationItem>,
    #[serde(default)]
    pub subordinates: Vec<SubordinateSpec>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SubordinateSpec {
    pub name: String,
    #[serde(default)]
    pub brief: String,
}

/// Result of popping the next queue item: either queue empty, leaf added (no LLM), or item to expand.
enum PopNext {
    Done,
    LeafProcessed,
    Item(QueueItem),
}

/// Pops the next queue item. If queue is empty returns Done. If front item is at depth >= depth_limit,
/// adds a leaf role and returns LeafProcessed. Otherwise removes and returns Item(item).
fn pop_next(state: &mut RoleGenState) -> PopNext {
    let item = match state.queue.first() {
        Some(q) => q.clone(),
        None => return PopNext::Done,
    };
    if state.depth_limit > 0 && item.depth >= state.depth_limit {
        state.queue.remove(0);
        let id = state.next_id();
        state.roles.push(Role {
            id,
            name: item.role_name,
            description: String::new(),
            background: String::new(),
            objectives: Vec::new(),
            skills: Vec::new(),
            skill_tree: Vec::new(),
            is_position_for_skill: None,
            collaboration: Vec::new(),
            parent_id: item.parent_id,
            subordinate_ids: Vec::new(),
        });
        return PopNext::LeafProcessed;
    }
    state.queue.remove(0);
    PopNext::Item(item)
}

/// Calls LLM for one role; returns raw response content. Optionally prints to stderr.
async fn call_llm_for_role(
    llm: &Arc<dyn langgraph::LlmClient>,
    system_prompt: &str,
    stream_print: bool,
    item: &QueueItem,
) -> Result<String, AgentError> {
    let user_prompt = format!(
        "针对角色「{}」生成描述与直接下属。仅输出一个 JSON 对象，格式见系统提示。",
        item.role_name
    );
    let messages = [
        Message::system(system_prompt),
        Message::user(user_prompt),
    ];
    let response = llm.invoke(&messages).await?;
    if stream_print {
        use std::io::Write;
        let _ = std::io::stderr().write_fmt(format_args!(
            "\n--- LLM [{}] ---\n{}\n---\n",
            item.role_name,
            response.content
        ));
        let _ = std::io::stderr().flush();
    }
    Ok(response.content)
}

/// Strips optional ```json ... ``` wrapper and returns the inner JSON string.
fn extract_json_from_response(content: &str) -> &str {
    let content = content.trim();
    content
        .strip_prefix("```json")
        .and_then(|s| s.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(content)
}

/// Parses LLM response JSON into LlmRoleOutput.
fn parse_llm_output(json_str: &str, raw_content: &str) -> Result<LlmRoleOutput, AgentError> {
    serde_json::from_str(json_str).map_err(|e| {
        AgentError::ExecutionFailed(format!("LLM output parse error: {}; raw: {}", e, raw_content))
    })
}

/// Resolves role id: if a placeholder with same name and parent exists, return (id, true); else (next_id(), false).
fn resolve_role_id(state: &RoleGenState, item: &QueueItem) -> (String, bool) {
    if let Some(placeholder) = state.roles.iter().find(|r| {
        r.name == item.role_name && r.parent_id == item.parent_id
    }) {
        (placeholder.id.clone(), true)
    } else {
        (state.next_id(), false)
    }
}

/// Creates subordinate placeholder roles and enqueues them (depth-first: insert at front, reversed).
fn enqueue_subordinates(
    state: &mut RoleGenState,
    out: &LlmRoleOutput,
    parent_id: &str,
    depth: u32,
) -> Vec<String> {
    let mut subordinate_ids = Vec::new();
    let new_items: Vec<QueueItem> = out
        .subordinates
        .iter()
        .map(|sub| QueueItem {
            role_name: sub.name.clone(),
            parent_id: Some(parent_id.to_string()),
            depth,
        })
        .collect();
    for sub in &out.subordinates {
        let sub_id = state.next_id();
        subordinate_ids.push(sub_id.clone());
        state.roles.push(Role {
            id: sub_id.clone(),
            name: sub.name.clone(),
            description: sub.brief.clone(),
            background: String::new(),
            objectives: Vec::new(),
            skills: Vec::new(),
            skill_tree: Vec::new(),
            is_position_for_skill: None,
            collaboration: Vec::new(),
            parent_id: Some(parent_id.to_string()),
            subordinate_ids: Vec::new(),
        });
    }
    for new_item in new_items.into_iter().rev() {
        state.queue.insert(0, new_item);
    }
    subordinate_ids
}

/// Builds skill tree from LLM output: use skill_tree if non-empty, else one leaf per flat skill.
fn normalize_skill_tree(out: &LlmRoleOutput) -> Vec<SkillNode> {
    if !out.skill_tree.is_empty() {
        return out.skill_tree.clone();
    }
    out.skills
        .iter()
        .filter(|s| !s.trim().is_empty())
        .map(|s| SkillNode {
            name: s.clone(),
            children: Vec::new(),
        })
        .collect()
}

/// Collects leaf skill names from the tree (skills with no children → 岗位).
fn collect_leaf_skills(tree: &[SkillNode]) -> Vec<String> {
    let mut leaves = Vec::new();
    for node in tree {
        if node.children.is_empty() {
            if !node.name.trim().is_empty() {
                leaves.push(node.name.clone());
            }
        } else {
            leaves.extend(collect_leaf_skills(&node.children));
        }
    }
    leaves
}

/// Flattens skill tree to a list (depth-first) for display.
fn flatten_skill_tree(tree: &[SkillNode]) -> Vec<String> {
    let mut out = Vec::new();
    for node in tree {
        if !node.name.trim().is_empty() {
            out.push(node.name.clone());
        }
        out.extend(flatten_skill_tree(&node.children));
    }
    out
}

/// Creates 岗位 roles for each leaf skill under this role and returns their ids.
fn create_positions_for_leaf_skills(
    state: &mut RoleGenState,
    parent_id: &str,
    leaf_skills: &[String],
) -> Vec<String> {
    let mut ids = Vec::new();
    for name in leaf_skills {
        let pos_id = state.next_id();
        ids.push(pos_id.clone());
        state.roles.push(Role {
            id: pos_id,
            name: format!("【岗位】{}", name),
            description: format!("专门负责技能：{}", name),
            background: String::new(),
            objectives: Vec::new(),
            skills: Vec::new(),
            skill_tree: Vec::new(),
            is_position_for_skill: Some(name.clone()),
            collaboration: Vec::new(),
            parent_id: Some(parent_id.to_string()),
            subordinate_ids: Vec::new(),
        });
    }
    ids
}

/// Updates existing placeholder role or pushes a new role. Adds 岗位 for leaf skills.
fn upsert_role(
    state: &mut RoleGenState,
    id: String,
    is_placeholder: bool,
    item: &QueueItem,
    out: LlmRoleOutput,
    mut org_subordinate_ids: Vec<String>,
) {
    let skill_tree = normalize_skill_tree(&out);
    let leaf_skills = collect_leaf_skills(&skill_tree);
    let position_ids = create_positions_for_leaf_skills(state, &id, &leaf_skills);
    org_subordinate_ids.extend(position_ids);
    let subordinate_ids = org_subordinate_ids;
    let skills = flatten_skill_tree(&skill_tree);

    if is_placeholder {
        if let Some(r) = state.roles.iter_mut().find(|r| r.id == id) {
            r.description = out.description;
            r.background = out.background;
            r.objectives = out.objectives;
            r.skills = skills;
            r.skill_tree = skill_tree;
            r.collaboration = out.collaboration;
            r.subordinate_ids = subordinate_ids;
        }
    } else {
        state.roles.push(Role {
            id,
            name: item.role_name.clone(),
            description: out.description,
            background: out.background,
            objectives: out.objectives,
            skills,
            skill_tree,
            is_position_for_skill: None,
            collaboration: out.collaboration,
            parent_id: item.parent_id.clone(),
            subordinate_ids,
        });
    }
}

/// Node that expands all queued roles in one run: for each queue item, calls
/// the LLM, parses JSON, appends the role and enqueues subordinates.
pub struct ExpandNode {
    llm: Arc<dyn langgraph::LlmClient>,
    /// System prompt loaded from prompt.md (or default).
    system_prompt: String,
    /// When true, print each LLM response to stderr as it is received.
    stream_print: bool,
}

impl ExpandNode {
    pub fn new(llm: Arc<dyn langgraph::LlmClient>, system_prompt: String) -> Self {
        Self {
            llm,
            system_prompt,
            stream_print: true,
        }
    }

    pub fn with_stream_print(mut self, on: bool) -> Self {
        self.stream_print = on;
        self
    }

    /// One expansion step: pop next (or add leaf), then optionally call LLM, parse, enqueue, upsert.
    async fn expand_one(&self, state: &mut RoleGenState) -> Result<bool, AgentError> {
        let item = match pop_next(state) {
            PopNext::Done => return Ok(false),
            PopNext::LeafProcessed => return Ok(true),
            PopNext::Item(it) => it,
        };

        let content = call_llm_for_role(
            &self.llm,
            &self.system_prompt,
            self.stream_print,
            &item,
        )
        .await?;
        let content_trimmed = content.trim();
        let json_str = extract_json_from_response(content_trimmed);
        let out = parse_llm_output(json_str, content_trimmed)?;

        let (id, is_placeholder) = resolve_role_id(&*state, &item);
        let subordinate_ids = enqueue_subordinates(state, &out, &id, item.depth + 1);
        upsert_role(state, id, is_placeholder, &item, out, subordinate_ids);

        Ok(true)
    }
}

#[async_trait]
impl Node<RoleGenState> for ExpandNode {
    fn id(&self) -> &str {
        "expand"
    }

    async fn run(&self, state: RoleGenState) -> Result<(RoleGenState, Next), AgentError> {
        let mut state = state;
        while self.expand_one(&mut state).await? {}
        Ok((state, Next::End))
    }
}

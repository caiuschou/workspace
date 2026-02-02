//! Role tree output: format to string or print to stdout.
//!
//! **Interaction**: Used by `main` for printing the result and by `check` for
//! building LLM check input. Single source of truth for tree layout (indent,
//! description, background, collaboration, objectives/skills, skill_tree).

use crate::state::{Role, SkillNode};

/// Formats skill tree to lines (indented by depth).
fn format_skill_tree_lines(tree: &[SkillNode], prefix: &str, depth: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let indent = "  ".repeat(depth);
    for node in tree {
        if node.name.trim().is_empty() {
            continue;
        }
        lines.push(format!("{}  Skill tree {}: {}", prefix, indent, node.name.trim()));
        lines.extend(format_skill_tree_lines(
            &node.children,
            prefix,
            depth + 1,
        ));
    }
    lines
}

/// Collects lines for one role and its subtree (recursive). Used by
/// `format_roles_to_text` and `print_role_tree`.
fn format_role_lines(roles: &[Role], id: &str, indent: u32) -> Vec<String> {
    let Some(r) = roles.iter().find(|x| x.id == id) else {
        return Vec::new();
    };
    let prefix = "  ".repeat(indent as usize);
    let mut lines = Vec::new();

    let role_label = match &r.is_position_for_skill {
        Some(s) => format!("{} (position·{})", r.name, s),
        None => r.name.clone(),
    };
    let desc = r.description.lines().next().unwrap_or("").trim();
    lines.push(format!("{}- {}: {}", prefix, role_label, desc));

    if !r.background.is_empty() {
        let bg = r.background.lines().next().unwrap_or("").trim();
        lines.push(format!("{}  Background: {}", prefix, bg));
    }
    for c in &r.collaboration {
        let contents = c.contents.join("; ");
        lines.push(format!(
            "{}  Collaboration with {}: {}",
            prefix,
            c.role.trim(),
            contents.trim()
        ));
    }
    if !r.skill_tree.is_empty() {
        lines.extend(format_skill_tree_lines(&r.skill_tree, &prefix, 0));
    } else if !r.objectives.is_empty() {
        for (i, obj) in r.objectives.iter().enumerate() {
            let sk = r.skills.get(i).map(|s| s.as_str()).unwrap_or("");
            lines.push(format!(
                "{}  - Objective: {}  → Skill: {}",
                prefix,
                obj.trim(),
                sk.trim()
            ));
        }
    } else if !r.skills.is_empty() {
        for sk in &r.skills {
            if !sk.trim().is_empty() {
                lines.push(format!("{}  Skill: {}", prefix, sk.trim()));
            }
        }
    }

    for sub_id in &r.subordinate_ids {
        lines.extend(format_role_lines(roles, sub_id, indent + 1));
    }
    lines
}

/// Formats the role tree to a single string (same structure as `print_role_tree`).
/// Used as input for the LLM check step.
pub fn format_roles_to_text(roles: &[Role], root_id: &str) -> String {
    format_role_lines(roles, root_id, 0).join("\n")
}

/// Prints the role tree to stdout.
pub fn print_role_tree(roles: &[Role], root_id: &str) {
    for line in format_role_lines(roles, root_id, 0) {
        println!("{}", line);
    }
}

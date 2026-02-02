//! LLM check: format role tree and ask LLM to review consistency/completeness.
//!
//! **Interaction**: Called by `main` after graph invoke. Uses `output::format_roles_to_text`
//! to build check input.

use langgraph::{LlmClient, Message};

use crate::output::format_roles_to_text;
use crate::state::Role;

const CHECK_SYSTEM: &str = "你是组织角色专家。请对以下生成的角色定义进行检查，输出检查结果：1）一致性（上下级、协同关系是否合理）2）完整性（目标、技能、协同是否齐全）3）是否符合明确、可达成、可衡量。直接输出检查结论与建议，不要重复原文。";

/// Runs the LLM check on the role tree: formats roles to text, sends to LLM,
/// prints result or error to stdout/stderr.
pub async fn run_check(roles: &[Role], root_id: &str, llm: &dyn LlmClient) {
    println!("\n========== 检查 ==========");
    println!("正在调用 LLM 检查生成结果...");
    let output_text = format_roles_to_text(roles, root_id);
    let messages = [
        Message::system(CHECK_SYSTEM),
        Message::user(format!("请检查以下角色定义：\n\n{}", output_text)),
    ];
    match llm.invoke(&messages).await {
        Ok(resp) => {
            println!(
                "检查结果：\n{}\n========== 检查结束 ==========",
                resp.content.trim()
            );
        }
        Err(e) => {
            eprintln!("检查失败（LLM 调用报错）: {}", e);
        }
    }
}

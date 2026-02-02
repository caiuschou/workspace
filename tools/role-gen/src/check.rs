//! LLM check: format role tree and ask LLM to review consistency/completeness.
//!
//! **Interaction**: Called by `main` after graph invoke. Uses `output::format_roles_to_text`
//! to build check input.

use langgraph::{LlmClient, Message};

use crate::output::format_roles_to_text;
use crate::state::Role;

const CHECK_SYSTEM: &str = "You are an organizational role expert. Please review the following role definitions and output: 1) Consistency (reporting lines, collaboration reasonable?) 2) Completeness (objectives, skills, collaboration complete?) 3) Whether they are specific, achievable, measurable. Output conclusions and suggestions directly, do not repeat the original text.";

/// Runs the LLM check on the role tree: formats roles to text, sends to LLM,
/// prints result or error to stdout/stderr.
pub async fn run_check(roles: &[Role], root_id: &str, llm: &dyn LlmClient) {
    println!("\n========== Check ==========");
    println!("Calling LLM to review generated results...");
    let output_text = format_roles_to_text(roles, root_id);
    let messages = [
        Message::system(CHECK_SYSTEM),
        Message::user(format!("Please review the following role definitions:\n\n{}", output_text)),
    ];
    match llm.invoke(&messages).await {
        Ok(resp) => {
            println!(
                "Check result:\n{}\n========== Check complete ==========",
                resp.content.trim()
            );
        }
        Err(e) => {
            eprintln!("Check failed (LLM invocation error): {}", e);
        }
    }
}

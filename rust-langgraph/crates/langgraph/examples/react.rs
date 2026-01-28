//! ReAct example: given a question like "3+5 equals?" or "10-2", runs ReActAgent with a
//! Calculator tool and prints the final answer.
//!
//! # Usage
//!
//! ```bash
//! cargo run -p langgraph --example react -- "3+5等于几"
//! cargo run -p langgraph --example react -- "10-2"
//! ```
//!
//! No API key is required: the example uses `SequenceMockLlmClient`, which returns predefined
//! strings in order (first "Action: calculator(...)", then "Answer: N").

use std::sync::Arc;

use langgraph::{AsyncAgent, CalculatorTool, ReActAgent, SequenceMockLlmClient, ToolRegistry};

fn main() {
    // Read the user question from the first CLI argument, or use a default.
    let query = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "3+5等于几".to_string());

    // Build the tool registry and register the calculator. ReActAgent will call it when the
    // mock LLM returns an "Action: calculator(...)" line.
    let mut reg = ToolRegistry::new();
    reg.register(Box::new(CalculatorTool::new()));
    let registry = Arc::new(reg);

    // SequenceMockLlmClient returns these two responses in order: first the "thought + action"
    // line (so the agent parses tool_calls and runs the calculator), then the "Answer: N" line
    // (so the agent stops and returns that answer). We derive the expression and numeric answer
    // from the query so that arbitrary expressions like "10-2" still work.
    let action_resp = format!(
        "Thought: I need to compute.\nAction: calculator({{\"expression\":\"{}\"}})\n",
        extract_expression_or_default(&query)
    );
    let answer = compute_answer(&query).unwrap_or_else(|| "8".to_string());
    let answer_resp = format!("Answer: {}", answer);

    let llm = SequenceMockLlmClient::new(vec![action_resp, answer_resp]);
    let agent = ReActAgent::new(llm, registry);

    // ReActAgent::run is async; we start a current-thread runtime and block until the run
    // completes.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    match rt.block_on(agent.run(query)) {
        Ok(out) => println!("{}", out),
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Extracts a numeric expression from the query string for use in the mock calculator call.
///
/// Keeps only ASCII digits and the operators `+ - * / .` and spaces; strips everything else.
/// If nothing remains, returns `"3+5"` so the mock still has a valid expression.
fn extract_expression_or_default(s: &str) -> String {
    let s = s.trim();
    let mut expr = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() || c == '+' || c == '-' || c == '*' || c == '/' || c == '.' || c == ' ' {
            expr.push(c);
        }
    }
    let expr = expr.replace(' ', "");
    if expr.is_empty() {
        "3+5".to_string()
    } else {
        expr
    }
}

/// Evaluates the expression (via `extract_expression_or_default`) and returns its result as a
/// string. Used to build the mock’s second reply ("Answer: N"). Returns `None` if evaluation
/// fails.
fn compute_answer(query: &str) -> Option<String> {
    let expr = extract_expression_or_default(query);
    if expr.is_empty() {
        return None;
    }
    evalexpr::eval(&expr).ok().map(|v| {
        if let Ok(i) = v.as_int() {
            i.to_string()
        } else if let Ok(f) = v.as_float() {
            f.to_string()
        } else {
            v.to_string()
        }
    })
}

//! ReAct (Reasoning + Acting) Agent: Think → Act (call tools) → Observe → loop until an answer.
//!
//! This crate provides the ReAct loop used by `ReActAgent`. The flow is:
//! - **Think**: Build prompt via `build_prompt`, call LLM via `LlmClient::chat`, parse output via `parse_thought`.
//! - **Act**: Execute parsed tool calls via `ToolRegistry::execute` (see `act_impl`).
//! - **Observe**: Format `ToolResult`s and append to history; then iterate or finish.
//!
//! Key types and functions:
//! - `ReActState`: Represents Thinking / Acting / Observing / Done (used for state-machine or step-wise APIs).
//! - `ParsedToolCall`: One tool invocation parsed from LLM text; convertible to `langgraph::ToolCall`.
//! - `build_prompt`: Builds the ReAct prompt from system prompt, tool descriptions, history, and query.
//! - `parse_thought`: Parses LLM output into `EitherAnswerOrCalls` (Answer or list of tool calls).
//! - `ReActAgent`: Holds `LlmClient` and `ToolRegistry`; implements `AsyncAgent` with Think/Act/Observe loop in `run`.
//!
//! Depends on the `langgraph` crate for `LlmClient`, `ToolRegistry`, `AsyncAgent`, and related types.

use std::sync::Arc;

use async_trait::async_trait;
use langgraph::llm::{ChatMessage, ChatRequest, LlmClient};
use langgraph::message::{ToolCall, ToolResult};
use langgraph::tool::{Tool, ToolRegistry};
use langgraph::{AgentError, AsyncAgent};

/// ReAct state: Think → Act → Observe → Done.
///
/// Used to represent the current phase of a ReAct loop. Interacts with:
/// - `ThinkNode`-style logic: `build_prompt`, `parse_thought`.
/// - `ActNode`-style logic: `ReActAgent::act_impl` and `ToolRegistry::execute`.
/// - `ObserveNode`-style logic: formatting `ToolResult`s and deciding to continue or finish.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ReActState {
    /// Thinking: user query, iteration count, and accumulated history (Thought/Action/Observation text).
    Thinking {
        /// User question; passed to `build_prompt` as `query`.
        query: String,
        /// Number of Think/Act/Observe rounds so far; compared to `ReActAgent::max_iterations` in `run`.
        iterations: usize,
        /// Accumulated "Thought: ... Action: ... Observation: ..." text from previous steps; passed to `build_prompt` as `history`.
        history: String,
    },
    /// Acting: parsed tool calls to be executed via `ToolRegistry::execute`.
    Acting {
        /// User question; passed to `build_prompt` as `query`.
        query: String,
        /// Number of Think/Act/Observe rounds so far; compared to `ReActAgent::max_iterations` in `run`.
        iterations: usize,
        /// Accumulated "Thought: ... Action: ... Observation: ..." text from previous steps; passed to `build_prompt` as `history`.
        history: String,
        /// Parsed tool invocations to run in this step; executed by `ReActAgent::act_impl` and `ToolRegistry::execute`.
        tool_calls: Vec<ParsedToolCall>,
    },
    /// Observing: tool execution results (see `ToolResult`).
    Observing {
        /// User question; passed to `build_prompt` as `query`.
        query: String,
        /// Number of Think/Act/Observe rounds so far; compared to `ReActAgent::max_iterations` in `run`.
        iterations: usize,
        /// Accumulated "Thought: ... Action: ... Observation: ..." text from previous steps; passed to `build_prompt` as `history`.
        history: String,
        /// Tool outputs from this round; appended to `history` as "Observation: ..." before the next Think step.
        results: Vec<ToolResult>,
    },
    /// Done: final answer string.
    Done {
        /// Final answer returned by the agent; produced when `parse_thought` returns `EitherAnswerOrCalls::Answer`.
        answer: String,
    },
}

/// A single tool call parsed from LLM output (e.g. "Action: tool_name({...})").
///
/// Produced by `parse_thought` and `parse_action_line`. Converted to `ToolCall` via `From<ParsedToolCall>`
/// when interoperating with `langgraph::ToolCall`. Consumed by `ReActAgent::act_impl` and `ToolRegistry::execute`.
#[derive(Debug, Clone)]
pub struct ParsedToolCall {
    /// Tool name, must match a tool registered in `ToolRegistry`.
    pub name: String,
    /// JSON string of arguments, parsed and passed to `Tool::execute` via `ToolRegistry::execute`.
    pub arguments: String,
}

/// Converts a parsed tool call into `langgraph::ToolCall` for use with message types.
impl From<ParsedToolCall> for ToolCall {
    fn from(p: ParsedToolCall) -> Self {
        ToolCall {
            name: p.name,
            arguments: p.arguments,
        }
    }
}

/// Default system prompt for the ReAct agent: instructs the model to output Thought / Action / Observation.
///
/// Used by `ReActAgent::new` and overridable via `ReActAgent::with_system_prompt`. The format is
/// parsed by `parse_thought` (Answer: / Action: tool_name(...)).
pub const DEFAULT_REACT_PROMPT: &str = r#"You are a reasoning agent. Answer the user's question step by step.
When you need to compute or use external tools, output exactly one line in this form:
Action: <tool_name>(<json arguments>)
Example: Action: calculator({"expression":"3+5"})
After you have enough information, output a line:
Answer: <final answer>
Use only the tools provided. One action per step."#;

/// Formats tool names and descriptions for inclusion in the ReAct prompt.
///
/// Consumes descriptions from `Tool::name()` and `Tool::description()`. Result is passed to
/// `build_prompt` as `tool_descriptions`. Used by `ReActAgent::tool_descriptions`.
pub fn format_tool_description(tools: &[&dyn Tool]) -> String {
    let mut lines: Vec<String> = tools
        .iter()
        .map(|t| format!("- {}: {}", t.name(), t.description()))
        .collect();
    lines.sort();
    lines.join("\n")
}

/// Builds the ReAct prompt string: system prompt + tool list + history + current question.
///
/// Used by `ReActAgent::run` each iteration. Expects:
/// - `system_prompt`: e.g. `DEFAULT_REACT_PROMPT` or custom from `with_system_prompt`.
/// - `tool_descriptions`: output of `format_tool_description` (from `tool_descriptions()`).
/// - `history`: accumulated Thought/Action/Observation text from previous steps.
/// - `query`: user question. The resulting string is sent as a user message to `LlmClient::chat`.
pub fn build_prompt(
    system_prompt: &str,
    tool_descriptions: &str,
    history: &str,
    query: &str,
) -> String {
    let mut s = system_prompt.to_string();
    s.push_str("\n\nAvailable tools:\n");
    s.push_str(tool_descriptions);
    if !history.is_empty() {
        s.push_str("\n\nPrevious steps:\n");
        s.push_str(history);
        s.push_str("\n\nContinue.");
    }
    s.push_str("\n\nQuestion: ");
    s.push_str(query);
    s
}

/// Parses LLM output into either a final answer or a list of tool calls.
///
/// Conventions (parsed line-by-line): line starting with `Answer:` → final answer; line starting with
/// `Action:` followed by `tool_name(...)` → one tool call (parsed via `parse_action_line`).
/// Returns the last `Answer` if present, otherwise all `Action` lines as `EitherAnswerOrCalls`.
/// Used by `ReActAgent::run` after each `LlmClient::chat` call.
pub fn parse_thought(text: &str) -> Result<EitherAnswerOrCalls, String> {
    let mut answer = None;
    let mut calls = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(a) = line.strip_prefix("Answer:") {
            answer = Some(a.trim().to_string());
            continue;
        }
        if let Some(rest) = line.strip_prefix("Action:") {
            let rest = rest.trim();
            if let Some((name, args)) = parse_action_line(rest) {
                calls.push(ParsedToolCall {
                    name: name.to_string(),
                    arguments: args.to_string(),
                });
            }
        }
    }

    if let Some(a) = answer {
        return Ok(EitherAnswerOrCalls::Answer(a));
    }
    if !calls.is_empty() {
        return Ok(EitherAnswerOrCalls::ToolCalls(calls));
    }
    Err("no Answer or Action line found".to_string())
}

/// Parses a single line of the form `tool_name(...)` into `(name, args_string)`.
///
/// Used internally by `parse_thought` for each "Action: ..." line. `args_string` is the content
/// between the parentheses (typically JSON), passed to `ParsedToolCall::arguments` and later to
/// `ToolRegistry::execute` after `serde_json::from_str`.
fn parse_action_line(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    let open = line.find('(')?;
    let name = line[..open].trim();
    let rest = line[open + 1..].trim_end();
    let close = rest.rfind(')')?;
    let args = rest[..close].trim();
    Some((name, args))
}

/// Result of parsing LLM output: either a final answer or a list of tool calls to execute.
///
/// Produced by `parse_thought`. Consumed by `ReActAgent::run`: `Answer` ends the loop and returns;
/// `ToolCalls` is executed via `act_impl` and results are appended to history for the next Think step.
#[derive(Debug)]
pub enum EitherAnswerOrCalls {
    /// Final answer string; agent stops and returns this.
    Answer(String),
    /// Tool calls to execute in this step; agent runs them and continues the loop.
    ToolCalls(Vec<ParsedToolCall>),
}

/// ReAct agent: holds an `LlmClient` and a `ToolRegistry`, implements `AsyncAgent` with a Think → Act → Observe loop.
///
/// - Uses `LlmClient::chat` for Think; `ToolRegistry::execute` (via `act_impl`) for Act; formats `ToolResult`s for Observe.
/// - Configurable via `with_system_prompt` and `with_max_iterations`. Implements `AsyncAgent::run`; see `langgraph::AsyncAgent`.
pub struct ReActAgent<C> {
    /// LLM client used in each Think step; `run` calls `LlmClient::chat` with the prompt from `build_prompt`.
    llm: C,
    /// Tool registry used by `tool_descriptions()` and `act_impl`; tools are looked up by name from parsed `Action:` lines.
    registry: Arc<ToolRegistry>,
    /// System prompt for the ReAct format; passed to `build_prompt` and overridable via `with_system_prompt`.
    system_prompt: String,
    /// Maximum Think/Act/Observe iterations; enforced at the start of each loop in `run`, see `with_max_iterations`.
    max_iterations: usize,
}

/// Debug format for `ReActAgent`: logs `llm`, `system_prompt`, `max_iterations`; registry shown as `"..."`
/// to avoid dumping tool internals.
impl<C: std::fmt::Debug> std::fmt::Debug for ReActAgent<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReActAgent")
            .field("llm", &self.llm)
            .field("registry", &"...")
            .field("system_prompt", &self.system_prompt)
            .field("max_iterations", &self.max_iterations)
            .finish()
    }
}

impl<C: LlmClient> ReActAgent<C> {
    /// Constructs a ReAct agent with the given LLM client and tool registry.
    ///
    /// Uses `DEFAULT_REACT_PROMPT` and `max_iterations = 10`. Customize via `with_system_prompt`
    /// and `with_max_iterations`. The registry is used by `tool_descriptions()` and `act_impl`.
    pub fn new(llm: C, registry: Arc<ToolRegistry>) -> Self {
        Self {
            llm,
            registry,
            system_prompt: DEFAULT_REACT_PROMPT.to_string(),
            max_iterations: 10,
        }
    }

    /// Sets the system prompt used in each Think step (see `build_prompt`).
    ///
    /// Overrides the default from `DEFAULT_REACT_PROMPT`. The prompt is passed to `build_prompt`
    /// as `system_prompt` when building the user message for `LlmClient::chat`.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// Sets the maximum number of Think/Act/Observe iterations before failing.
    ///
    /// Checked at the start of each loop in `run`. If exceeded, returns
    /// `AgentError::ExecutionFailed("max ReAct iterations exceeded")`.
    pub fn with_max_iterations(mut self, n: usize) -> Self {
        self.max_iterations = n;
        self
    }

    /// Collects tool names and descriptions from the registry for the ReAct prompt.
    ///
    /// Uses `ToolRegistry::names` and `ToolRegistry::get`, then `format_tool_description`.
    /// Result is passed to `build_prompt` as `tool_descriptions` in each Think step.
    fn tool_descriptions(&self) -> String {
        let names = self.registry.names();
        let tools: Vec<&dyn Tool> = names
            .iter()
            .filter_map(|n| self.registry.get(n))
            .collect();
        format_tool_description(&tools)
    }

    /// Executes a batch of parsed tool calls via the registry and returns a list of `ToolResult`s.
    ///
    /// For each `ParsedToolCall`: parses `arguments` with `serde_json::from_str`, calls
    /// `ToolRegistry::execute(name, args)`, and builds a `ToolResult` with `name` and stringified
    /// content. Errors from the registry are wrapped in `AgentError::ExecutionFailed`.
    fn act_impl(&self, calls: &[ParsedToolCall]) -> Result<Vec<ToolResult>, AgentError> {
        let mut results = Vec::with_capacity(calls.len());
        for call in calls {
            let args = serde_json::from_str(&call.arguments).unwrap_or(serde_json::json!({}));
            let out = self
                .registry
                .execute(&call.name, args)
                .map_err(|e| AgentError::ExecutionFailed(e.to_string()))?;
            let content = if let Some(n) = out.as_i64() {
                n.to_string()
            } else if let Some(f) = out.as_f64() {
                f.to_string()
            } else {
                out.to_string()
            };
            results.push(ToolResult {
                name: call.name.clone(),
                content,
            });
        }
        Ok(results)
    }
}

#[async_trait]
impl<C: LlmClient + Send + Sync> AsyncAgent for ReActAgent<C> {
    type Input = String;
    type Output = String;
    type Error = AgentError;

    /// Returns the agent identifier `"ReActAgent"` (see `langgraph::AsyncAgent`).
    fn name(&self) -> &str {
        "ReActAgent"
    }

    /// Runs the ReAct loop: Think (build_prompt → LlmClient::chat → parse_thought), Act (act_impl),
    /// Observe (append ToolResult to history). Stops when parse_thought returns Answer or when
    /// max_iterations is exceeded. Input is the user question; output is the final answer string.
    async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let query = input;
        let tool_desc = self.tool_descriptions();
        let mut history = String::new();
        let mut iterations = 0usize;

        loop {
            if iterations >= self.max_iterations {
                return Err(AgentError::ExecutionFailed("max ReAct iterations exceeded".into()));
            }
            let prompt = build_prompt(&self.system_prompt, &tool_desc, &history, &query);
            let req = ChatRequest {
                messages: vec![ChatMessage::user(&prompt)],
                temperature: Some(0.0),
                max_tokens: Some(1024),
            };
            let resp = self
                .llm
                .chat(req)
                .await
                .map_err(|e| AgentError::ExecutionFailed(e.to_string()))?;
            let text = resp.content;

            match parse_thought(&text) {
                Ok(EitherAnswerOrCalls::Answer(a)) => return Ok(a),
                Ok(EitherAnswerOrCalls::ToolCalls(calls)) => {
                    history.push_str("\nThought: ");
                    history.push_str(&text);
                    history.push_str("\nAction: ");
                    for c in &calls {
                        history.push_str(&format!("{}({}) ", c.name, c.arguments));
                    }
                    let results = self.act_impl(&calls)?;
                    let obs: String = results
                        .iter()
                        .map(|r| format!("{} => {}", r.name, r.content))
                        .collect::<Vec<_>>()
                        .join(", ");
                    history.push_str("\nObservation: ");
                    history.push_str(&obs);
                    iterations += 1;
                }
                Err(e) => {
                    return Err(AgentError::ExecutionFailed(format!(
                        "parse_thought failed: {}",
                        e
                    )))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests for `parse_action_line` and `parse_thought` (ReAct parsing).

    use super::*;

    #[test]
    fn parse_action_line_normal() {
        let (name, args) = parse_action_line("calculator({\"expression\":\"3+5\"})").unwrap();
        assert_eq!(name, "calculator");
        assert_eq!(args, "{\"expression\":\"3+5\"}");
    }

    #[test]
    fn parse_thought_answer() {
        let r = parse_thought("foo\nAnswer: 8\nbar").unwrap();
        assert!(matches!(r, EitherAnswerOrCalls::Answer(a) if a == "8"));
    }

    #[test]
    fn parse_thought_action() {
        let r = parse_thought("Thought: I need to compute.\nAction: calculator({\"expression\":\"3+5\"})").unwrap();
        match &r {
            EitherAnswerOrCalls::ToolCalls(c) => {
                assert_eq!(c.len(), 1);
                assert_eq!(c[0].name, "calculator");
                assert_eq!(c[0].arguments, "{\"expression\":\"3+5\"}");
            }
            _ => panic!("expected ToolCalls"),
        }
    }
}

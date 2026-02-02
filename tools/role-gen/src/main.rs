//! CLI for role-gen: build the graph, invoke with root role, print result.
//!
//! Loads config (API keys, prompt path) via `role_gen::Config`; system prompt
//! from `prompt.md` or default.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_openai::config::OpenAIConfig;
use clap::Parser;
use langgraph::{ChatOpenAI, StateGraph, START, END};

use role_gen::{resolve_prompt_path, ExpandNode, QueueItem, RoleGenState};

#[derive(Parser)]
#[command(name = "role-gen")]
#[command(about = "Generate a role tree from a root role using LLM (OpenAI)")]
struct Args {
    /// Root role name to expand (e.g. CEO)
    #[arg(default_value = "CEO")]
    root_role: String,

    /// Maximum depth to expand (0 = no limit). Depth 0 = root; roles at depth >= this are added as leaves (no LLM). Default 10 = up to 11 levels.
    #[arg(long, short, default_value = "10")]
    depth_limit: u32,

    /// OpenAI model name
    #[arg(long, default_value = "gpt-4o-mini")]
    model: String,

    /// Path to prompt.md (system prompt). Default: crate dir / prompt.md or PROMPT_PATH env
    #[arg(short, long)]
    prompt: Option<PathBuf>,

    /// Do not print LLM responses to stderr
    #[arg(long)]
    no_stream_print: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let args = Args::parse();

    let cfg = role_gen::Config::from_env_and_args(
        resolve_prompt_path(args.prompt.clone()),
        args.root_role.clone(),
        args.depth_limit,
        args.model.clone(),
        args.no_stream_print,
    )?;

    let openai_config = OpenAIConfig::new()
        .with_api_base(cfg.api_base.clone())
        .with_api_key(cfg.api_key.clone());
    let llm: Arc<dyn langgraph::LlmClient> = Arc::new(
        ChatOpenAI::with_config(openai_config, cfg.model.clone()).with_temperature(0.3),
    );
    let expand = Arc::new(
        ExpandNode::new(llm.clone(), cfg.system_prompt.clone())
            .with_stream_print(!cfg.no_stream_print),
    );

    let mut graph = StateGraph::<RoleGenState>::new();
    graph.add_node("expand", expand);
    graph.add_edge(START, "expand");
    graph.add_edge("expand", END);

    let compiled = graph.compile()?;

    let initial = RoleGenState {
        roles: Vec::new(),
        queue: vec![QueueItem {
            role_name: cfg.root_role.clone(),
            parent_id: None,
            depth: 0,
        }],
        depth_limit: cfg.depth_limit,
    };

    let final_state = compiled
        .invoke(initial, None)
        .await
        .map_err(|e| anyhow::anyhow!("invoke failed: {}", e))?;

    println!("Root role: {}", cfg.root_role);
    println!("Depth limit: {}", cfg.depth_limit);
    println!("Roles: {}", final_state.roles.len());
    println!();
    if let Some(root) = final_state.roles.iter().find(|r| r.parent_id.is_none()) {
        role_gen::print_role_tree(&final_state.roles, &root.id);
    }

    let root_id = final_state
        .roles
        .iter()
        .find(|r| r.parent_id.is_none())
        .map(|r| r.id.as_str())
        .unwrap_or("");
    role_gen::run_check(&final_state.roles, root_id, llm.as_ref()).await;


    Ok(())
}

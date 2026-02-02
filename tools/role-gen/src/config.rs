//! Configuration: API keys, prompt path, and resolved system prompt.
//!
//! **Interaction**: Used by `main` to build LLM client and expand node.
//! Loads from environment and CLI args via `Config::from_env_and_args`.

use std::path::PathBuf;

use anyhow::{Context, Result};

/// Default system prompt when prompt.md is missing.
pub const DEFAULT_PROMPT: &str = "You are an expert at defining organizational roles.\nFor the given role, output a JSON object only, no other text:\n{\"description\": \"...\", \"subordinates\": [{\"name\": \"...\", \"brief\": \"...\"}, ...]}\nUse \"subordinates\" for direct reports only. If none, use \"subordinates\": [].";

/// Resolved configuration: API base/key, model, system prompt, and run options.
#[derive(Clone, Debug)]
pub struct Config {
    pub api_key: String,
    pub api_base: String,
    pub model: String,
    pub system_prompt: String,
    pub depth_limit: u32,
    pub root_role: String,
    pub no_stream_print: bool,
}

impl Config {
    /// Resolves config from environment and CLI. Caller should run
    /// `dotenv::dotenv().ok()` before this. `prompt_path` is the path to
    /// prompt.md (e.g. from `resolve_prompt_path(args.prompt)`).
    pub fn from_env_and_args(
        prompt_path: PathBuf,
        root_role: String,
        depth_limit: u32,
        model: String,
        no_stream_print: bool,
    ) -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY not set (put it in .env or environment)")?
            .trim()
            .to_string();
        let api_base = std::env::var("OPENAI_API_BASE")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string())
            .trim()
            .to_string();
        if api_key.is_empty() {
            anyhow::bail!(
                "OPENAI_API_KEY is empty (check .env: no spaces, whole key on one line)"
            );
        }

        let system_prompt = if prompt_path.exists() {
            std::fs::read_to_string(&prompt_path).context("read prompt.md")?
        } else {
            DEFAULT_PROMPT.to_string()
        };

        Ok(Self {
            api_key,
            api_base,
            model,
            system_prompt,
            depth_limit,
            root_role,
            no_stream_print,
        })
    }
}

/// Resolves the path to prompt.md: `--prompt` > PROMPT_PATH env > crate dir / prompt.md.
pub fn resolve_prompt_path(prompt_arg: Option<PathBuf>) -> PathBuf {
    prompt_arg
        .or_else(|| std::env::var("PROMPT_PATH").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("prompt.md"))
}

# langgraph-react

ReAct (Reasoning + Acting) agent for [langgraph]: Think → Act (call tools) → Observe → loop until an answer.

Depends on `langgraph` for `LlmClient`, `ToolRegistry`, `AsyncAgent`, and related types.

## Usage

```toml
[dependencies]
langgraph = { path = "../langgraph" }
langgraph-react = { path = "../langgraph-react" }
```

```rust
use std::sync::Arc;
use langgraph::{AsyncAgent, ToolRegistry};
use langgraph_react::ReActAgent;

// Build registry, then:
// let agent = ReActAgent::new(llm, registry);
// let out = agent.run(query).await?;
```

## Example

```bash
cargo run -p langgraph-react --example react -- "3+5等于几"
cargo run -p langgraph-react --example react -- "10-2"
```

[langgraph]: ../langgraph

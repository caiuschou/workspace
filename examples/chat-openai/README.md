# ChatOpenAI Example

Demonstrates invoking OpenAI Chat Completions API using `ChatOpenAI` from rust-langgraph.

## Prerequisites

Set the `OPENAI_API_KEY` environment variable:

```bash
export OPENAI_API_KEY="sk-..."
```

## Usage

```bash
# Default prompt
cargo run -p chat-openai-example

# Custom prompt
cargo run -p chat-openai-example -- "What is Rust programming language?"

# Multi-word prompt (use quotes)
cargo run -p chat-openai-example -- "Explain async/await in Rust in 3 sentences"
```

## Code Overview

```rust
use langgraph::{ChatOpenAI, LlmClient, Message};

// Build messages
let messages = vec![
    Message::system("You are a helpful assistant."),
    Message::user("Hello!"),
];

// Create client (reads OPENAI_API_KEY from env)
let client = ChatOpenAI::new("gpt-4o-mini");

// Invoke (single-call, aligns with LangChain's invoke)
let response = client.invoke(&messages).await?;
println!("{}", response.content);
```

## API Alignment

| LangChain | rust-langgraph |
|-----------|----------------|
| `ChatOpenAI` | `ChatOpenAI` |
| `invoke(input)` | `invoke(&messages)` |
| `AIMessage` | `LlmResponse { content, tool_calls }` |

## See Also

- [14-chat-openai.md](../../docs/rust-langgraph/14-chat-openai.md) - Full design doc
- [llm/README.md](../../rust-langgraph/crates/langgraph/src/llm/README.md) - LLM module docs

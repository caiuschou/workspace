# ChatZhipu Example

Demonstrates invoking Zhipu (智谱) GLM Chat Completions API using `ChatZhipu` from rust-langgraph.

## Prerequisites

Create a `.env` file in the example directory (or project root) with your API key. Get it from [智谱开放平台](https://open.bigmodel.cn/):

```bash
# examples/chat-zhipu/.env or project root .env
ZHIPU_API_KEY=your-api-key
```

Or set the environment variable: `export ZHIPU_API_KEY="your-api-key"`.

## Usage

```bash
# Default prompt (Chinese)
cargo run -p chat-zhipu-example

# Custom prompt
cargo run -p chat-zhipu-example -- "用三句话解释什么是 Rust 语言"
```

## Models

Supported models (from Zhipu open platform):

- `glm-4-flash` - Fast, cost-effective
- `glm-4-plus` - More capable
- `glm-4-long` - Long context

## Code Overview

```rust
use langgraph::{ChatZhipu, LlmClient, Message};

let messages = vec![
    Message::system("你是一个有帮助的助手。"),
    Message::user("你好！"),
];

let client = ChatZhipu::new("glm-4-flash");
let response = client.invoke(&messages).await?;
println!("{}", response.content);
```

## See Also

- [14-chat-openai.md](../../docs/rust-langgraph/14-chat-openai.md) - LLM design doc
- [llm/README.md](../../rust-langgraph/crates/langgraph/src/llm/README.md) - LLM module docs

# role-gen

LLM-based role tree generator: given a root role (e.g. CEO), expands it into a full hierarchy using OpenAI.

## Config

Copy `.env.example` to `.env` in `tools/role-gen` (or workspace root) and set:

- `OPENAI_API_BASE` — OpenAI-compatible API base URL (e.g. `https://gptproto.com/v1`; no trailing slash).
- `OPENAI_API_KEY` — API key.

`.env` is gitignored; do not commit real keys.

## Build and run

From workspace root or from `tools/role-gen`:

```bash
cargo build -p role-gen --release
cargo run -p role-gen -- [OPTIONS] [ROOT_ROLE]
```

## Usage

```text
role-gen [OPTIONS] [ROOT_ROLE]

Arguments:
  [ROOT_ROLE]  Root role name to expand (e.g. CEO) [default: CEO]

Options:
  -d, --depth-limit <DEPTH_LIMIT>  Max depth to expand (0 = no limit). Depth 0 = root; roles at depth >= this are leaves. Default 10 = 11 levels. [default: 10]
  -m, --model <MODEL>              OpenAI model name [default: gpt-4o-mini]
  -p, --prompt <PROMPT>            Path to prompt.md (system prompt). Default: crate dir / prompt.md or PROMPT_PATH env
      --no-stream-print            Do not print each LLM response to stderr
  -h, --help                       Print help
```

- **Prompt**: System prompt is read from `prompt.md` in the crate directory (or `--prompt path` / `PROMPT_PATH`). Edit `prompt.md` to change the LLM instructions.
- **Stream print**: By default each LLM response is printed to stderr as it is received (`--- LLM [role] ---` … `---`). Use `--no-stream-print` to disable.

- **Depth**: `depth_limit` is 0-based (root = 0). We call the LLM for depth 0..=depth_limit-1; roles at depth >= depth_limit are added as leaves. So `-d 10` gives 11 levels (0..=10) with levels 0–9 expanded by LLM.

Example:

```bash
cargo run -p role-gen -- CTO
```

## Design

- **State**: `RoleGenState` with `roles`, `queue`, `depth_limit`. See [docs/role-gen/spec.md](../../docs/role-gen/spec.md).
- **Graph**: Single node `expand` that runs an internal loop: pop from queue, call LLM (ChatOpenAI), parse JSON, append roles and enqueue subordinates, until queue is empty or depth limit.
- **LLM**: langgraph-rust `ChatOpenAI` (feature `openai`); `OPENAI_API_BASE` and `OPENAI_API_KEY` from `.env` or environment.

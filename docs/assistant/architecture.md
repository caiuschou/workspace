# Assistant 架构设计

> Assistant 的详细架构设计和模块说明

## 系统架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Presentation Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │     CLI      │  │     Web      │  │   Desktop    │             │
│  │   (Rust)     │  │  (React)     │  │   (Tauri)    │             │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘             │
└─────────┼──────────────────┼──────────────────┼───────────────────┘
          │                  │                  │
          └──────────────────┼──────────────────┘
                             │
┌────────────────────────────▼───────────────────────────────────────┐
│                          Core Engine                               │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                     Session Manager                          │   │
│  │  - 会话生命周期管理  - 上下文维护  - 状态持久化              │   │
│  └────────────────────┬────────────────────────────────────────┘   │
│  ┌────────────────────▼────────────────────────────────────────┐   │
│  │                    Message Pipeline                          │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │   │
│  │  │  Input   │→│ Intent   │→│  Route   │→│ Response│    │   │
│  │  │ Parser   │  │ Analyzer │  │  Engine  │  │ Formatter│   │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │   │
│  └─────────────────────────────────────────────────────────────┘   │
└────────────────────────────┬──────────────────────────────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
┌────────▼─────────┐ ┌──────▼──────┐ ┌──────────▼──────────┐
│ Personality      │ │   Memory    │ │  Agora Integration  │
│ Engine           │ │   Manager   │ │                     │
│  - Prompt 模板   │ │  - 短期记忆 │ │  - Agent 发现      │
│  - 风格转换      │ │  - 会话存储 │ │  - 消息路由        │
│  - Tone 控制     │ │  - 长期记忆 │ │  - 结果聚合        │
└──────────────────┘ └─────────────┘ └─────────────────────┘
```

## 核心模块

### 1. Session Manager

管理用户会话的生命周期和状态。

```rust
pub struct SessionManager {
    sessions: HashMap<SessionId, Session>,
    storage: Storage,
    config: SessionConfig,
}

impl SessionManager {
    pub async fn create(&mut self, title: String) -> Result<Session>;
    pub async fn get(&self, id: &SessionId) -> Option<Session>;
    pub async fn list(&self) -> Vec<Session>;
    pub async fn delete(&mut self, id: &SessionId) -> Result<()>;
    pub async fn append_message(&mut self, id: &SessionId, msg: Message) -> Result<()>;
}

pub struct Session {
    pub id: SessionId,
    pub title: String,
    pub profile: Profile,
    pub messages: Vec<Message>,
    pub context: Context,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**职责**：
- 会话创建、获取、删除
- 消息历史管理
- 上下文窗口控制（自动裁剪超长历史）
- 会话持久化

### 2. Message Pipeline

消息处理管道，处理用户输入到响应输出的完整流程。

```
用户输入
    │
    ▼
┌─────────────────┐
│  Input Parser   │  解析文本、代码块、@mention、文件引用
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Intent Analyzer │  分析意图：闲聊 / 任务执行 / Agent 调用
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌────────┐ ┌──────────────┐
│ Direct │ │ Agent Route  │
│  LLM   │ │  (via Agora) │
└───┬────┘ └──────┬───────┘
    │             │
    │     ┌───────▼────────┐
    │     │  Agent Results │
    │     │  Aggregator    │
    │     └───────┬────────┘
    │             │
    └──────┬──────┘
           ▼
┌─────────────────┐
│Response Formatter│  应用助手类型风格，格式化输出
└────────┬────────┘
         │
         ▼
    返回响应
```

### 3. Intent Analyzer

意图分析器，决定如何处理用户输入。

```rust
pub enum Intent {
    Chat,           // 普通对话，直接使用 LLM
    AgentCall {     // 需要调用 Agent
        agent: String,
        task: String,
    },
    MultiAgent {    // 需要多个 Agent 协作
        agents: Vec<String>,
        task: String,
    },
    System {        // 系统命令
        command: SystemCommand,
    },
}

pub struct IntentAnalyzer {
    llm: LLMClient,
    agent_registry: AgentRegistry,
}

impl IntentAnalyzer {
    pub async fn analyze(&self, input: &str) -> Intent;
    pub fn detect_mentions(&self, input: &str) -> Vec<String>;
}
```

### 4. Router

路由引擎，将请求分发到合适的处理单元。

```rust
pub struct Router {
    agora: AgoraClient,
    llm: LLMClient,
    agent_map: HashMap<String, AgentInfo>,
}

impl Router {
    pub async fn route(&self, intent: Intent, input: &str) -> Response;
    pub async fn call_agent(&self, agent: &str, task: &str) -> AgentResult;
    pub async fn call_llm(&self, messages: Vec<Message>) -> LLMResponse;
}
```

### 5. Personality Engine

助手类型引擎，管理不同的对话风格。

```rust
pub struct PersonalityEngine {
    profiles: HashMap<String, Profile>,
    current: String,
}

pub struct Profile {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    pub tone: Tone,
    pub response_style: ResponseStyle,
}

pub enum Tone {
    Professional,
    Casual,
    Creative,
    Concise,
}

pub enum ResponseStyle {
    Structured,      // 使用标题、列表、表格
    Conversational,  // 自然对话流
    Detailed,        // 详细解释
    Brief,           // 简洁回答
}
```

详见 [助手类型配置](./personality.md)。

### 6. Memory Manager

记忆管理器，负责存储和检索对话历史。

```rust
pub struct MemoryManager {
    short_term: ShortTermMemory,
    session_store: SessionStore,
    long_term: LongTermMemory,
}

pub struct ShortTermMemory {
    context_window: usize,
    current_messages: Vec<Message>,
}

pub struct SessionStore {
    storage: Storage,
    max_sessions: usize,
}

pub struct LongTermMemory {
    vector_store: VectorStore,
    extractor: KnowledgeExtractor,
}
```

详见 [记忆管理](./memory.md)。

### 7. Agora Integration

Agora 集成层，处理与其他 Agent 的通信。

```rust
pub struct AgoraIntegration {
    client: AgoraClient,
    agent_registry: AgentRegistry,
    result_cache: Cache,
}

impl AgoraIntegration {
    pub async fn discover_agents(&self) -> Vec<AgentInfo>;
    pub async fn call_agent(&self, agent: &str, task: &Task) -> AgentResult;
    pub async fn publish_status(&self, status: Status);
    pub async fn subscribe_events(&self, space: &str);
}
```

## 数据流

### 对话流程

```
┌─────────────────────────────────────────────────────────────────┐
│                         对话流程                                │
└─────────────────────────────────────────────────────────────────┘

User: "帮我分析 src/ 目录下的代码"

    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ 1. Input Parser                                                 │
│    - 解析输入: "分析 src/ 目录"                                  │
│    - 检测引用: src/ 目录                                         │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Intent Analyzer                                              │
│    - 分析意图: 需要代码分析能力                                  │
│    - 匹配 Agent: Explore Agent (只读搜索)                       │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. Router → Agora                                              │
│    - 加入 space: "task.{task_id}"                               │
│    - 发布任务: { type: "analyze", path: "src/" }                │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. Explore Agent 处理                                           │
│    - Glob 查找所有文件                                          │
│    - Grep 搜索关键模式                                          │
│    - Read 读取文件内容                                          │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. Agora 返回结果                                               │
│    - space.event: { from: "explore", results: [...] }           │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ 6. Response Formatter                                          │
│    - 应用当前助手风格                                           │
│    - 生成结构化回复                                             │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
Assistant: "我分析了 src/ 目录，发现以下结构..."
```

### Agent 协作流程

```
┌─────────────────────────────────────────────────────────────────┐
│                      Agent 协作流程                             │
└─────────────────────────────────────────────────────────────────┘

User: "@coder 重构 auth 模块，然后 @reviewer 审查代码"

    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ 1. Intent Analyzer                                             │
│    - 检测 @mentions: @coder, @reviewer                          │
│    - 确定顺序: coder → reviewer (串行)                          │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Router: 创建任务 Space                                       │
│    - space.join("task.001")                                    │
│    - 发布步骤: [{ agent: "coder", task: "..." }]                │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. Coder Agent 执行                                            │
│    - space.publish("task.001", { progress: 0.5 })               │
│    - 完成后: { from: "coder", result: "refactored code" }        │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. Reviewer Agent 执行                                         │
│    - 接收 Coder 的结果                                         │
│    - space.publish("task.001", { progress: 0.8 })               │
│    - 完成后: { from: "reviewer", result: "review comments" }    │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. 聚合结果                                                    │
│    - 收集所有 Agent 的输出                                      │
│    - 生成综合回复                                               │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
Assistant: "重构完成！Coder 改进了登录逻辑，
            Reviewer 建议加强密码验证..."
```

## 错误处理

```rust
pub enum AssistantError {
    // Agora 相关
    AgoraConnectionFailed,
    AgentNotFound(String),
    AgentTimeout(String),

    // LLM 相关
    LLMError(String),
    RateLimited,

    // 记忆相关
    StorageError(String),
    ContextOverflow,

    // 解析相关
    InvalidInput(String),
    IntentAnalysisFailed,
}

impl AssistantError {
    pub fn user_message(&self) -> String;
    pub fn should_retry(&self) -> bool;
}
```

## 性能考虑

| 策略 | 说明 |
|------|------|
| **流式响应** | LLM 响应实时流式返回，减少延迟感 |
| **并行 Agent** | 独立任务并行执行，聚合结果 |
| **上下文压缩** | 超长历史自动摘要压缩 |
| **结果缓存** | 相同查询返回缓存结果 |
| **连接池** | Agora 连接复用，减少握手开销 |

## 扩展点

| 扩展点 | 说明 |
|--------|------|
| **自定义 Profile** | 添加新的助手类型 |
| **自定义 Intent** | 扩展意图识别逻辑 |
| **自定义 Agent** | 注册新的专业 Agent |
| **自定义 Storage** | 替换存储后端 |
| **自定义 LLM** | 支持新的 LLM 提供商 |

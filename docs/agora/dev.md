# Agora 开发文档

## 技术栈

| 组件 | 技术 |
|------|------|
| 服务端 | Rust + Axum + tokio-tungstenite |
| 协议 | JSON-RPC 2.0 + 扩展事件类型 |
| 客户端 SDK | TypeScript |

## 项目结构

```
crates/
├── agora-core/          # 核心类型和逻辑
│   ├── src/
│   │   ├── lib.rs
│   │   ├── agent.rs     # Agent 类型定义
│   │   ├── space.rs     # Space 管理逻辑
│   │   ├── event.rs     # 事件类型定义
│   │   ├── protocol.rs  # 协议常量
│   │   └── error.rs     # 错误类型
│   └── Cargo.toml
│
├── agora-server/        # WebSocket 服务端
│   ├── src/
│   │   ├── main.rs
│   │   ├── websocket/
│   │   │   ├── mod.rs
│   │   │   ├── handler.rs    # WebSocket 处理器
│   │   │   └── state.rs      # 服务端状态
│   │   ├── router.rs         # 事件路由
│   │   └── auth.rs           # 认证（可选）
│   └── Cargo.toml
│
└── mcp-server/          # 扩展现有 MCP Server
    └── src/websocket/   # 复用现有 WebSocket 基础设施

packages/
└── agora-sdk/           # TypeScript 客户端 SDK
    ├── src/
    │   ├── index.ts
    │   ├── client.ts    # AgoraClient 主类
    │   ├── space.ts     # Space API
    │   ├── event.ts     # 事件类型定义
    │   └── error.ts     # 错误处理
    ├── package.json
    └── tsconfig.json
```

## 核心数据结构

### Agent

```rust
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub type AgentId = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentType {
    Process,
    Thread,
    Distributed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub agent_type: AgentType,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub connection_id: Option<String>,
    pub spaces: HashSet<SpaceId>,
}
```

### Space

```rust
pub type SpaceId = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpaceType {
    Public,
    Private,
    System,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RetentionPolicy {
    Volatile,     // 不保留历史
    Persistent,   // 保留历史
}

#[derive(Clone, Debug)]
pub struct Space {
    pub id: SpaceId,
    pub space_type: SpaceType,
    pub members: HashSet<AgentId>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub retention: RetentionPolicy,
    pub history: VecDeque<AgoraEvent>,
}
```

### 事件

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgoraEvent {
    #[serde(rename = "agent.register")]
    AgentRegister { id: String, agent: AgentInfo },
    #[serde(rename = "agent.registered")]
    AgentRegistered { id: String, agent: AgentInfo },
    #[serde(rename = "agent.heartbeat")]
    AgentHeartbeat { timestamp: i64 },
    #[serde(rename = "space.join")]
    SpaceJoin { id: String, space: SpaceId },
    #[serde(rename = "space.joined")]
    SpaceJoined { id: String, space: SpaceId, members: Vec<AgentId> },
    #[serde(rename = "space.leave")]
    SpaceLeave { id: String, space: SpaceId },
    #[serde(rename = "space.publish")]
    SpacePublish { id: String, space: SpaceId, data: serde_json::Value },
    #[serde(rename = "space.event")]
    SpaceEvent { space: SpaceId, data: serde_json::Value },
    #[serde(rename = "space.list")]
    SpaceList { id: String },
    #[serde(rename = "space.list")]
    SpaceListResult { id: String, spaces: Vec<SpaceInfo> },
    #[serde(rename = "error")]
    Error { code: String, message: String, request_id: Option<String> },
}
```

## 服务端状态管理

```rust
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Clone)]
pub struct AgoraState {
    pub agents: Arc<RwLock<HashMap<AgentId, Agent>>>,
    pub spaces: Arc<RwLock<HashMap<SpaceId, Space>>>,
    pub connections: Arc<RwLock<HashMap<String, AgentId>>>, // connection_id -> agent_id
}

impl AgoraState {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            spaces: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_agent(&self, agent: Agent, connection_id: String) -> Result<()> {
        let mut agents = self.agents.write().await;
        let mut connections = self.connections.write().await;
        agents.insert(agent.id.clone(), agent);
        connections.insert(connection_id, agent.id);
        Ok(())
    }

    pub async fn join_space(&self, agent_id: &AgentId, space_id: &SpaceId) -> Result<()> {
        let mut spaces = self.spaces.write().await;
        let space = spaces.entry(space_id.clone()).or_insert_with(|| {
            Space::new(space_id.clone(), SpaceType::Public)
        });
        space.members.insert(agent_id.clone());
        Ok(())
    }

    pub async fn publish(&self, space_id: &SpaceId, data: serde_json::Value) -> Result<()> {
        let spaces = self.spaces.read().await;
        if let Some(space) = spaces.get(space_id) {
            // 广播给所有成员
            for member_id in &space.members {
                // 发送消息...
            }
        }
        Ok(())
    }
}
```

## TypeScript SDK 设计

### AgoraClient

```typescript
export interface AgoraClientConfig {
  url: string;
  agentId: string;
  token?: string;
  capabilities?: string[];
  metadata?: Record<string, any>;
  reconnect?: boolean;
}

export class AgoraClient {
  private ws: WebSocket | null = null;
  private config: AgoraClientConfig;
  private eventHandlers: Map<string, EventHandler[]> = new Map();
  private pendingEvents: Map<string, PendingEvent> = new Map();

  constructor(config: AgoraClientConfig) {
    this.config = config;
  }

  // 连接
  async connect(): Promise<void> { }

  // 断开
  disconnect(): void { }

  // Space API
  readonly space = {
    join: (spaceId: string): Promise<void> => { },
    leave: (spaceId: string): Promise<void> => { },
    publish: (spaceId: string, data: any): Promise<void> => { },
    list: (): Promise<SpaceInfo[]> => { },
  };

  // 事件监听
  on(event: string, handler: EventHandler): void { }
  off(event: string, handler?: EventHandler): void { }

  // 状态
  get isConnected(): boolean { }
  get currentSpaces(): string[] { }
}
```

### 增量消息处理

```typescript
interface PendingEvent {
  eventId: string;
  space: string;
  accumulator: any;      // 累积的数据
  startTime: number;
}

export class AgoraClient {
  // ...

  private handleDelta(event: DeltaEvent): void {
    const { event_id, space, data } = event;

    if (!this.pendingEvents.has(event_id)) {
      // 第一个 delta，创建累积器
      this.pendingEvents.set(event_id, {
        eventId: event_id,
        space,
        accumulator: data,
        startTime: Date.now(),
      });
    } else {
      // 后续 delta，累积数据
      const pending = this.pendingEvents.get(event_id)!;
      pending.accumulator = this.merge(pending.accumulator, data);
    }
  }

  private handleDone(event: DoneEvent): void {
    const { event_id } = event;
    const pending = this.pendingEvents.get(event_id);

    if (pending) {
      // 触发完整事件回调
      this.emit('space.event', {
        space: pending.space,
        data: pending.accumulator,
      });

      // 清理
      this.pendingEvents.delete(event_id);
    }
  }

  private merge(acc: any, delta: any): any {
    // 文本拼接
    if (delta.text && acc.text) {
      return { ...acc, text: acc.text + delta.text };
    }
    // 其他字段合并
    return { ...acc, ...delta };
  }

  // 超时清理
  private cleanupPendingEvents(): void {
    const now = Date.now();
    const TIMEOUT = 30000; // 30 秒

    for (const [id, event] of this.pendingEvents) {
      if (now - event.startTime > TIMEOUT) {
        this.pendingEvents.delete(id);
      }
    }
  }
}
```

### 使用示例（支持增量）

```typescript
const client = new AgoraClient({
  url: 'ws://localhost:8080/ws',
  agentId: 'agent-001'
});

// 监听事件（自动处理增量合并）
client.on('space.event', (event) => {
  console.log(`[${event.space}]`, event.data);
  // event.data 已经是完整的合并后数据
});

// 手动处理增量（如需更精细控制）
client.on('space.event.delta', (event) => {
  console.log('Delta:', event.data);
});

client.on('space.event.done', (event) => {
  console.log('Done:', event.event_id);
});

// 取消正在传输的事件
await client.space.cancelEvent('evt_001');
```

## 实现阶段

### Phase 1: MVP

- [ ] `agora-core` - 核心类型定义
- [ ] `agora-server` - WebSocket 服务端
  - [ ] 连接管理
  - [ ] Agent 注册/注销
  - [ ] Space 创建/加入/离开
  - [ ] 消息路由和广播
- [ ] `agora-sdk` - TypeScript 客户端
  - [ ] WebSocket 连接
  - [ ] 事件发送/接收
  - [ ] Space API

### Phase 2: 增强

- [ ] JWT 认证
- [ ] 消息历史（Persistent Space）
- [ ] Agent 元数据管理
- [ ] 速率限制

### Phase 3: 生产

- [ ] 监控与指标（Prometheus）
- [ ] 负载均衡
- [ ] 水平扩展支持（Redis Pub/Sub）
- [ ] 健康检查

## 参考资料

- [OpenAI Realtime API](https://platform.openai.com/docs/guides/realtime)
- 现有 MCP WebSocket 实现 (`crates/mcp-server/src/websocket/`)
- JSON-RPC 2.0 规范

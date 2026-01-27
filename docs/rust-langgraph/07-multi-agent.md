# 多 Agent 协作

类型安全的多 Agent 协作系统。

## 消息传递

```rust
use tokio::sync::{mpsc, oneshot};

/// Agent 消息
pub enum AgentMessage {
    /// 任务请求
    Task {
        input: serde_json::Value,
        respond: oneshot::Sender<Result<serde_json::Value, AgentError>>,
    },

    /// 状态查询
    Status {
        respond: oneshot::Sender<AgentStatus>,
    },

    /// 停止信号
    Shutdown,
}

/// Agent 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Busy,
    Error,
    Stopped,
}

/// Actor 风格的 Agent
pub struct ActorAgent {
    name: String,
    tx: mpsc::Sender<AgentMessage>,
}

impl ActorAgent {
    /// 启动 Agent
    pub fn spawn<A>(name: String, agent: A) -> Self
    where
        A: Agent + Send + 'static,
        A::Input: DeserializeOwned + Send,
        A::Output: Serialize + Send,
    {
        let (tx, mut rx) = mpsc::channel(64);

        tokio::spawn(async move {
            let mut agent = agent;
            let mut status = AgentStatus::Idle;

            while let Some(msg) = rx.recv().await {
                match msg {
                    AgentMessage::Task { input, respond } => {
                        status = AgentStatus::Busy;

                        let input = match serde_json::from_value(input) {
                            Ok(i) => i,
                            Err(e) => {
                                let _ = respond.send(Err(AgentError::InvalidInput));
                                status = AgentStatus::Error;
                                continue;
                            }
                        };

                        let result = agent.run(input).await;

                        let output = result.map(|v| {
                            serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
                        });

                        let _ = respond.send(output);
                        status = AgentStatus::Idle;
                    }
                    AgentMessage::Status { respond } => {
                        let _ = respond.send(status);
                    }
                    AgentMessage::Shutdown => {
                        break;
                    }
                }
            }
        });

        Self { name, tx }
    }

    /// 发送任务
    pub async fn send_task(
        &self,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, AgentError> {
        let (tx, rx) = oneshot::channel();

        self.tx
            .send(AgentMessage::Task { input, respond: tx })
            .await
            .map_err(|_| AgentError::ChannelClosed)?;

        rx.await
            .map_err(|_| AgentError::ChannelClosed)?
    }

    /// 查询状态
    pub async fn status(&self) -> Result<AgentStatus, AgentError> {
        let (tx, rx) = oneshot::channel();

        self.tx
            .send(AgentMessage::Status { respond: tx })
            .await
            .map_err(|_| AgentError::ChannelClosed)?;

        rx.await
            .map_err(|_| AgentError::ChannelClosed)
    }
}
```

## Supervisor 模式

```rust
/// Supervisor - 协调多个 Worker
pub struct Supervisor {
    workers: Vec<ActorAgent>,
    strategy: RoutingStrategy,
}

/// 路由策略
pub enum RoutingStrategy {
    /// 轮询
    RoundRobin(usize),

    /// 最少负载
    LeastLoaded,

    /// 一致性哈希
    ConsistentHash,

    /// 指定名称
   ByName(String),
}

impl Supervisor {
    pub fn new(strategy: RoutingStrategy) -> Self {
        Self {
            workers: Vec::new(),
            strategy,
        }
    }

    pub fn add_worker(&mut self, worker: ActorAgent) {
        self.workers.push(worker);
    }

    /// 分发任务
    pub async fn dispatch(
        &mut self,
        task: serde_json::Value,
    ) -> Result<serde_json::Value, AgentError> {
        let worker = self.select_worker().await?;
        worker.send_task(task).await
    }

    /// 选择 Worker
    async fn select_worker(&mut self) -> Result<&ActorAgent, AgentError> {
        if self.workers.is_empty() {
            return Err(AgentError::NoWorkers);
        }

        match &mut self.strategy {
            RoutingStrategy::RoundRobin(idx) => {
                let worker = &self.workers[*idx % self.workers.len()];
                *idx += 1;
                Ok(worker)
            }
            RoutingStrategy::LeastLoaded => {
                let mut best = 0;
                let mut best_load = usize::MAX;

                for (i, worker) in self.workers.iter().enumerate() {
                    match worker.status().await {
                        Ok(AgentStatus::Idle) => return Ok(&self.workers[i]),
                        Ok(_) => {}
                        Err(_) => continue,
                    }
                }

                Ok(&self.workers[best])
            }
            RoutingStrategy::ByName(name) => {
                self.workers
                    .iter()
                    .find(|w| w.name == *name)
                    .ok_or_else(|| AgentError::WorkerNotFound(name.clone()))
            }
            RoutingStrategy::ConsistentHash => {
                // 简化实现：使用第一个
                Ok(&self.workers[0])
            }
        }
    }
}
```

## 专用 Worker Trait

```rust
/// Worker trait
#[async_trait]
pub trait Worker: Send + Sync {
    /// 能力描述
    fn capabilities(&self) -> &[&str];

    /// 处理任务
    async fn handle(&self, task: Task) -> Result<TaskOutput, WorkerError>;
}

/// 任务
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub kind: String,
    pub input: serde_json::Value,
}

/// 任务输出
#[derive(Debug, Clone)]
pub struct TaskOutput {
    pub task_id: String,
    pub result: serde_json::Value,
}

/// 研究员 Worker
pub struct ResearcherWorker {
    llm: Arc<dyn LlmClient>,
}

#[async_trait]
impl Worker for ResearcherWorker {
    fn capabilities(&self) -> &[&str] {
        &["search", "research", "gather"]
    }

    async fn handle(&self, task: Task) -> Result<TaskOutput, WorkerError> {
        let prompt = format!("Research: {}", task.input);
        let result = self.llm.complete(&prompt).await?;

        Ok(TaskOutput {
            task_id: task.id,
            result: serde_json::json!({ "research": result }),
        })
    }
}

/// 分析师 Worker
pub struct AnalystWorker {
    llm: Arc<dyn LlmClient>,
}

#[async_trait]
impl Worker for AnalystWorker {
    fn capabilities(&self) -> &[&str] {
        &["analyze", "summarize", "report"]
    }

    async fn handle(&self, task: Task) -> Result<TaskOutput, WorkerError> {
        let prompt = format!("Analyze: {}", task.input);
        let result = self.llm.complete(&prompt).await?;

        Ok(TaskOutput {
            task_id: task.id,
            result: serde_json::json!({ "analysis": result }),
        })
    }
}
```

## 工作流编排

```rust
/// 工作流步骤
#[derive(Debug, Clone)]
pub enum Step {
    /// 串行执行
    Serial(Vec<Step>),

    /// 并行执行
    Parallel(Vec<Step>),

    /// 条件分支
    Branch {
        condition: String,
        true_branch: Box<Step>,
        false_branch: Box<Step>,
    },

    /// Worker 任务
    Task {
        worker: String,
        input: serde_json::Value,
    },

    /// 等待
    Delay(Duration),

    /// 完成
    Done,
}

/// 工作流执行器
pub struct WorkflowExecutor {
    workers: HashMap<String, ActorAgent>,
}

impl WorkflowExecutor {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
        }
    }

    pub fn register_worker(&mut self, name: String, worker: ActorAgent) {
        self.workers.insert(name, worker);
    }

    /// 执行工作流
    pub async fn execute(&self, step: Step) -> Result<serde_json::Value, WorkflowError> {
        match step {
            Step::Serial(steps) => {
                let mut result = serde_json::Value::Null;
                for s in steps {
                    result = self.execute(s).await?;
                }
                Ok(result)
            }
            Step::Parallel(steps) => {
                let futures: Vec<_> = steps
                    .into_iter()
                    .map(|s| self.execute(s))
                    .collect();

                let results = futures::future::join_all(futures).await;

                // 合并结果
                let mut output = serde_json::json!({});
                for (i, r) in results.into_iter().enumerate() {
                    output[i] = r?;
                }
                Ok(output)
            }
            Step::Task { worker, input } => {
                let agent = self
                    .workers
                    .get(&worker)
                    .ok_or_else(|| WorkflowError::WorkerNotFound(worker))?;

                agent.send_task(input).await.map_err(Into::into)
            }
            Step::Delay(duration) => {
                tokio::time::sleep(duration).await;
                Ok(serde_json::Value::Null)
            }
            Step::Done => Ok(serde_json::Value::Null),
            Step::Branch { .. } => {
                // 条件分支实现
                Ok(serde_json::Value::Null)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Worker not found: {0}")]
    WorkerNotFound(String),

    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
}
```

## 多 Agent 协作示例

```rust
/// 研究任务 Agent 组
pub struct ResearchTeam {
    supervisor: Supervisor,
    executor: WorkflowExecutor,
}

impl ResearchTeam {
    pub async fn new(llm: Arc<dyn LlmClient>) -> Result<Self, AgentError> {
        let mut supervisor = Supervisor::new(RoutingStrategy::LeastLoaded);
        let mut executor = WorkflowExecutor::new();

        // 创建研究员
        let researcher = ActorAgent::spawn(
            "researcher".to_string(),
            ResearcherWorker { llm: llm.clone() },
        );
        supervisor.add_worker(researcher.clone());
        executor.register_worker("researcher".to_string(), researcher);

        // 创建分析师
        let analyst = ActorAgent::spawn(
            "analyst".to_string(),
            AnalystWorker { llm: llm.clone() },
        );
        supervisor.add_worker(analyst.clone());
        executor.register_worker("analyst".to_string(), analyst);

        // 创建写作者
        let writer = ActorAgent::spawn(
            "writer".to_string(),
            WriterWorker { llm },
        );
        supervisor.add_worker(writer.clone());
        executor.register_worker("writer".to_string(), writer);

        Ok(Self { supervisor, executor })
    }

    /// 执行研究任务
    pub async fn research(&mut self, topic: String) -> Result<String, AgentError> {
        let workflow = Step::Parallel(vec![
            Step::Task {
                worker: "researcher".to_string(),
                input: serde_json::json!({ "topic": topic }),
            },
        ]);

        self.executor.execute(workflow).await?;
        Ok("Research complete".to_string())
    }
}

struct WriterWorker {
    llm: Arc<dyn LlmClient>,
}

#[async_trait]
impl Worker for WriterWorker {
    fn capabilities(&self) -> &[&str] {
        &["write", "draft", "format"]
    }

    async fn handle(&self, task: Task) -> Result<TaskOutput, WorkerError> {
        let prompt = format!("Write: {}", task.input);
        let result = self.llm.complete(&prompt).await?;

        Ok(TaskOutput {
            task_id: task.id,
            result: serde_json::json!({ "content": result }),
        })
    }
}
```

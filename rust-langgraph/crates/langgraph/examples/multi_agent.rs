//! 多 Agent 雏形示例：Supervisor + Worker，发 Task 收 TaskResult。
//!
//! 运行：`cargo run -p langgraph --example multi_agent -- "你好"`
//! 演示：创建 EchoWorker、AgentChannel、ActorAgent、Supervisor，派发一个 Task 并打印结果。
//!
//! 流程概览：
//! 1. 构造 Worker（本示例为 EchoWorker）并用 WorkerActor 包装成 Handler
//! 2. 建信道、拆成 ActorRef 与 Receiver，构造 ActorAgent
//! 3. 在后台任务中跑 ActorAgent 的消息循环
//! 4. Supervisor 持 Router 与 Worker 的 ActorRef，对 Task 做 dispatch（内部走 request-响应）
//! 5. 取得结果后 drop Supervisor，关闭信道，Actor 退出，join 结束

use std::sync::Arc;

use langgraph::{
    ActorAgent, ActorId, AgentChannel, EchoWorker, RoundRobinRouter, Supervisor, Task,
    WorkerActor,
};

#[tokio::main]
async fn main() {
    // 从命令行取首个参数作为任务载荷，缺省为 "hello"
    let payload = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "hello".to_string());

    // --- 1) Worker 与 Handler ---
    // EchoWorker 实现 Worker trait，对 Task 原样返回 payload；WorkerActor 将其包装成 Handler<()>，供 ActorAgent 调用
    let worker = Arc::new(EchoWorker::new());
    let handler = WorkerActor::new(worker);

    // --- 2) 信道与 Actor ---
    // AgentChannel 提供收发两端；split 得到 ActorRef（发给调用方）和 Receiver（交给 ActorAgent 消费）
    let ch = AgentChannel::new(32);
    let (actor_ref, rx) = ch.split();
    let mut agent = ActorAgent::new(
        ActorId::new("echo-1"),
        rx,
        (),   // 本示例状态为空
        handler,
    );

    // --- 3) 在后台跑消息循环 ---
    // ActorAgent::run() 从 inbox 取消息并交给 Handler 处理，直到收到 Stop 或信道关闭
    let handle = tokio::spawn(async move { agent.run().await });

    // --- 4) Supervisor 派发任务 ---
    // Router 决定选哪个 Worker（本例仅一个 Worker，RoundRobin 仍可用）；dispatch 内部对所选 Worker 做 request(task)，等待 TaskResult
    let router = RoundRobinRouter::new();
    let mut supervisor = Supervisor::new(router);
    supervisor.add_worker(actor_ref);

    let task = Task::new(&payload);
    match supervisor.dispatch(task).await {
        Ok(res) => println!("success={} output={}", res.success, res.output),
        Err(e) => {
            eprintln!("dispatch error: {e}");
            std::process::exit(1);
        }
    }

    // --- 5) 收尾：关闭信道，等待 Actor 退出 ---
    //  drop Supervisor 会 drop 其持有的 ActorRef，发送端关闭，Agent 的 recv() 得到 None 后 run() 返回
    drop(supervisor);
    if let Err(e) = handle.await {
        eprintln!("actor join error: {e}");
    }
}

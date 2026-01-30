//! Integration tests for StateGraph: compile validation and invoke.

use async_trait::async_trait;
use langgraph::{Agent, AgentError, CompilationError, Message, StateGraph};

#[derive(Debug, Clone, Default)]
struct AgentState {
    pub messages: Vec<Message>,
}

struct EchoAgent;

impl EchoAgent {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Agent for EchoAgent {
    fn name(&self) -> &str {
        "echo"
    }
    type State = AgentState;
    async fn run(&self, state: Self::State) -> Result<Self::State, AgentError> {
        let mut messages = state.messages;
        if let Some(Message::User(s)) = messages.last() {
            messages.push(Message::Assistant(s.clone()));
        }
        Ok(AgentState { messages })
    }
}

#[tokio::test]
async fn compile_fails_when_edge_refers_to_unknown_node() {
    let mut graph = StateGraph::<AgentState>::new();
    graph.add_node("echo", Box::new(EchoAgent::new()));
    graph.add_edge("echo");
    graph.add_edge("missing");

    match graph.compile() {
        Err(CompilationError::NodeNotFound(id)) => assert_eq!(id, "missing"),
        _ => panic!("expected NodeNotFound"),
    }
}

#[tokio::test]
async fn invoke_single_node_chain() {
    let mut graph = StateGraph::<AgentState>::new();
    graph
        .add_node("echo", Box::new(EchoAgent::new()))
        .add_edge("echo");

    let compiled = graph.compile().unwrap();
    let mut state = AgentState::default();
    state.messages.push(Message::User("hi".into()));

    let state = compiled.invoke(state, None).await.unwrap();
    let last = state.messages.last().unwrap();
    assert!(matches!(last, Message::Assistant(s) if s == "hi"));
}

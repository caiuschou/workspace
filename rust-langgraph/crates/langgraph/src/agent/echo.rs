//! Echo Agent：输入即回显，用于 Sprint 1 最小可运行示例。

use crate::error::AgentError;
use crate::traits::Agent;

/// Echo Agent：`run` 将输入原样返回。
#[derive(Debug, Default)]
pub struct EchoAgent;

impl EchoAgent {
    /// 构造新的 EchoAgent。
    pub fn new() -> Self {
        Self
    }
}

impl Agent for EchoAgent {
    type Input = String;
    type Output = String;
    type Error = AgentError;

    fn name(&self) -> &str {
        "EchoAgent"
    }

    fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_returns_input() {
        let agent = EchoAgent::new();
        let out = agent.run("你好".to_string()).unwrap();
        assert_eq!(out, "你好");
    }

    #[test]
    fn name_is_echo_agent() {
        let agent = EchoAgent::new();
        assert_eq!(agent.name(), "EchoAgent");
    }
}

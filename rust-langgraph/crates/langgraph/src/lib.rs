//! LangGraph 风格的类型安全 Agent 与状态机。
//!
//! 开发计划见仓库内 `docs/rust-langgraph/ROADMAP.md`。

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

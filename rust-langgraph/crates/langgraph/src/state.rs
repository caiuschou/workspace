//! 状态机与执行器。
//!
//! - `StateMachine`: 定义状态、事件、输出与单步转换
//! - `StateTransition`: 转换结果（继续 / 产出输出 / 结束）
//! - `Runner`: 持有机器与当前状态，按事件迭代并限制步数

use crate::error::StateError;

/// 单步转换结果：继续新状态、产出输出并保留状态、或结束并产出输出。
///
/// 与 `StateMachine::transition` 配合使用。
#[derive(Debug, Clone)]
pub enum StateTransition<S, O> {
    /// 继续到新状态，未产出最终输出。
    Continue(S),
    /// 产出中间或最终输出，并保留状态（可用于多轮产出）。
    Output(O, S),
    /// 结束，产出最终输出。
    Done(O),
}

/// 状态机 trait。
///
/// 泛型参数：
/// - `S`: 状态类型
/// - `E`: 事件类型
/// - `O`: 输出类型
///
/// 与 `Runner` 配合实现 ReAct 等循环：`Runner::run(events)` 迭代调用 `transition`，
/// 直到得到 `Done` 或超过步数限制。
pub trait StateMachine {
    /// 状态类型。
    type State: Clone + Send + Sync;
    /// 事件类型。
    type Event: Send;
    /// 输出类型。
    type Output: Send;

    /// 单步转换：给定当前状态与事件，返回转换结果或错误。
    fn transition(
        &self,
        state: Self::State,
        event: Self::Event,
    ) -> Result<StateTransition<Self::State, Self::Output>, StateError>;
}

/// 步数限制的默认值。
pub const DEFAULT_MAX_STEPS: usize = 32;

/// 状态机执行器：持有机器与当前状态，按事件序列迭代转换并限制最大步数。
///
/// 与 `StateMachine` 配合使用。`run` 从初始状态开始，依次消费 `events` 并调用
/// `StateMachine::transition`；遇到 `Done` 即返回，遇到 `Output` 可收集（本实现
/// 以最后一次 `Output` 或 `Done` 的值为最终结果）；超过 `max_steps` 返回
/// `StateError::MaxStepsExceeded`。
#[derive(Debug)]
pub struct Runner<M>
where
    M: StateMachine,
{
    /// 状态机。
    machine: M,
    /// 最大步数。
    max_steps: usize,
}

impl<M> Runner<M>
where
    M: StateMachine,
{
    /// 使用给定状态机与最大步数构造。
    pub fn new(machine: M, max_steps: usize) -> Self {
        Self { machine, max_steps }
    }

    /// 使用默认最大步数构造。
    pub fn with_default_limit(machine: M) -> Self {
        Self::new(machine, DEFAULT_MAX_STEPS)
    }

    /// 从初始状态开始，按事件迭代执行，返回最终输出或错误。
    ///
    /// `events` 需产生 `StateMachine::Event`；每消费一个事件调用一次 `transition`。
    /// 若某次 `transition` 返回 `Done(o)`，则立即返回 `Ok(o)`；
    /// 若返回 `Output(o, s)`，则更新状态并继续（本实现以最后一次产出为准）；
    /// 若返回 `Continue(s)`，则仅更新状态。
    /// 若步数超过 `max_steps`，返回 `Err(StateError::MaxStepsExceeded(max_steps))`。
    pub fn run<I>(&self, initial: M::State, events: I) -> Result<M::Output, StateError>
    where
        I: IntoIterator<Item = M::Event>,
    {
        let mut state = initial;
        let mut last_output: Option<M::Output> = None;

        for (steps, event) in events.into_iter().enumerate() {
            if steps >= self.max_steps {
                return Err(StateError::MaxStepsExceeded(self.max_steps));
            }
            match self.machine.transition(state.clone(), event) {
                Ok(StateTransition::Continue(s)) => {
                    state = s;
                }
                Ok(StateTransition::Output(o, s)) => {
                    last_output = Some(o);
                    state = s;
                }
                Ok(StateTransition::Done(o)) => {
                    return Ok(o);
                }
                Err(e) => return Err(e),
            }
        }

        last_output.ok_or_else(|| StateError::InvalidTransition("no output produced".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 最小状态机：状态=计数，事件=+1，输出=计数；到 3 则 Done。
    struct CounterMachine;

    impl StateMachine for CounterMachine {
        type State = u32;
        type Event = ();
        type Output = u32;

        fn transition(
            &self,
            state: Self::State,
            _event: Self::Event,
        ) -> Result<StateTransition<Self::State, Self::Output>, StateError> {
            let next = state + 1;
            if next >= 3 {
                Ok(StateTransition::Done(next))
            } else {
                Ok(StateTransition::Output(next, next))
            }
        }
    }

    #[test]
    fn runner_returns_done_output() {
        let r = Runner::new(CounterMachine, 10);
        let events = [(), (), ()];
        let out = r.run(0u32, events).unwrap();
        assert_eq!(out, 3);
    }

    #[test]
    fn runner_max_steps() {
        struct NeverDone;
        impl StateMachine for NeverDone {
            type State = ();
            type Event = ();
            type Output = ();

            fn transition(
                &self,
                _s: (),
                _e: (),
            ) -> Result<StateTransition<(), ()>, StateError> {
                Ok(StateTransition::Continue(()))
            }
        }
        let r = Runner::new(NeverDone, 2);
        let events = [(), (), (), ()];
        let res = r.run((), events);
        assert!(matches!(res, Err(StateError::MaxStepsExceeded(2))));
    }
}

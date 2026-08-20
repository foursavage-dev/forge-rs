#![forbid(unsafe_code)]

use crate::executor::{ExecutorError, TaskExecutor};
use crate::task::{Task, TaskOutcome};

pub trait TaskMiddleware: Send + Sync {
    fn pre_execute(&self, _task: &mut Task) -> Result<(), ExecutorError> {
        Ok(())
    }

    fn post_execute(&self, _task: &Task, _outcome: &mut TaskOutcome) -> Result<(), ExecutorError> {
        Ok(())
    }
}

pub struct MiddlewareChainExecutor<E> {
    inner: E,
    middlewares: Vec<Box<dyn TaskMiddleware>>,
}

impl<E: TaskExecutor> MiddlewareChainExecutor<E> {
    pub fn new(inner: E) -> Self {
        Self {
            inner,
            middlewares: Vec::new(),
        }
    }

    pub fn with_middleware(mut self, middleware: Box<dyn TaskMiddleware>) -> Self {
        self.middlewares.push(middleware);
        self
    }
}

impl<E: TaskExecutor> TaskExecutor for MiddlewareChainExecutor<E> {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        let mut modified_task = task.clone();
        for m in &self.middlewares {
            m.pre_execute(&mut modified_task)?;
        }

        let mut outcome = self.inner.execute(&modified_task)?;

        for m in self.middlewares.iter().rev() {
            m.post_execute(&modified_task, &mut outcome)?;
        }

        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandSpec;
    use crate::task::TaskStatus;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    struct DummyExecutor;
    impl TaskExecutor for DummyExecutor {
        fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
            Ok(TaskOutcome {
                status: TaskStatus::Executed,
                exit_code: Some(0),
                stdout: format!("ran {}", task.label),
                stderr: String::new(),
                duration: Duration::from_millis(10),
            })
        }
    }

    struct FlagMiddleware {
        pre_called: Arc<AtomicBool>,
        post_called: Arc<AtomicBool>,
    }

    impl TaskMiddleware for FlagMiddleware {
        fn pre_execute(&self, _task: &mut Task) -> Result<(), ExecutorError> {
            self.pre_called.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn post_execute(
            &self,
            _task: &Task,
            _outcome: &mut TaskOutcome,
        ) -> Result<(), ExecutorError> {
            self.post_called.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_middleware_chain_execution() {
        let pre_flag = Arc::new(AtomicBool::new(false));
        let post_flag = Arc::new(AtomicBool::new(false));

        let middleware = FlagMiddleware {
            pre_called: Arc::clone(&pre_flag),
            post_called: Arc::clone(&post_flag),
        };

        let executor =
            MiddlewareChainExecutor::new(DummyExecutor).with_middleware(Box::new(middleware));

        let mut spec = CommandSpec::new("echo");
        spec.args.push("hello".to_string());
        let task = Task::new("build_test", "test task", spec);
        let outcome = executor.execute(&task).unwrap();

        assert_eq!(outcome.status, TaskStatus::Executed);
        assert!(pre_flag.load(Ordering::SeqCst));
        assert!(post_flag.load(Ordering::SeqCst));
    }
}

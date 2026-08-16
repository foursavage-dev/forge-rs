#![forbid(unsafe_code)]

pub mod command;
pub mod executor;
pub mod task;
pub mod async_executor;

pub use command::CommandSpec;
pub use executor::{ExecutorError, ProcessExecutor, TaskExecutor};
pub use task::{CacheEntry, Task, TaskOutcome, TaskStatus};
pub use async_executor::AsyncProcessExecutor;

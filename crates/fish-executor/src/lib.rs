#![forbid(unsafe_code)]

pub mod async_executor;
pub mod command;
pub mod cow;
pub mod executor;
pub mod linker;
pub mod middleware;
pub mod response_file;
pub mod task;

pub use async_executor::AsyncProcessExecutor;
pub use command::CommandSpec;
pub use cow::{CloneStrategy, KernelCowCloner};
pub use executor::{ExecutorError, ProcessExecutor, TaskExecutor};
pub use linker::{LinkerDispatcher, LinkerKind};
pub use middleware::{MiddlewareChainExecutor, TaskMiddleware};
pub use response_file::ResponseFileWriter;
pub use task::{CacheEntry, Task, TaskOutcome, TaskStatus};

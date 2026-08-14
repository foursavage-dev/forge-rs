//! `forge-executor`: the task model and task executors.
//!
//! This crate defines what a unit of work looks like ([`Task`]) and the
//! boundary ([`TaskExecutor`]) separating the scheduler from whatever runs
//! tasks: local processes ([`ProcessExecutor`]), or cached wrappers built on
//! top of `forge-cache`.
//!
//! `CommandSpec` is a pure, displayable recipe for a process; it never
//! touches the OS until an executor runs it.

#![forbid(unsafe_code)]

pub mod command;
pub mod executor;
pub mod task;

pub use command::CommandSpec;
pub use executor::{ExecutorError, ProcessExecutor, TaskExecutor};
pub use task::{CacheEntry, Task, TaskOutcome, TaskStatus};

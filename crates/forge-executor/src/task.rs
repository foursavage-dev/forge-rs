//! Task model shared by the scheduler, cache, and backends.

use std::time::Duration;

use crate::command::CommandSpec;

/// A fingerprint associates a cacheable task with the inputs it depends on.
///
/// The scheduler does not interpret it; the caching executor uses
/// `(key, fingerprint)` to answer "did this exact input already build?".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    /// Namespace for the cached input fingerprint (e.g. crate name + mode).
    pub key: String,
    /// A stable hash of all task inputs.
    pub fingerprint: String,
}

/// One unit of work in a build.
///
/// Backends construct these from a package graph. Tasks are immutable after
/// construction: scheduling state lives on the graph, not in the task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    /// Short human-readable name, used in progress output.
    pub label: String,
    /// Longer human-readable description, shown in task output / failures.
    pub description: String,
    /// The command to run in order to build this task.
    pub spec: CommandSpec,
    /// Optional cache key + fingerprint for incremental builds.
    pub cache: Option<CacheEntry>,
}

impl Task {
    /// Create a new task with no cache entry.
    pub fn new(
        label: impl Into<String>,
        description: impl Into<String>,
        spec: CommandSpec,
    ) -> Self {
        Self {
            label: label.into(),
            description: description.into(),
            spec,
            cache: None,
        }
    }

    /// Attach a cache entry to this task.
    pub fn with_cache(mut self, entry: CacheEntry) -> Self {
        self.cache = Some(entry);
        self
    }
}

/// What happened when a task ran (or didn't need to run).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// The task's command was executed and exited successfully.
    Executed,
    /// The task was cacheable and its cached fingerprint matched.
    Cached,
    /// The task failed (nonzero exit code, execute error, or executor panic).
    Failed,
}

/// The result of attempting to execute one task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOutcome {
    pub status: TaskStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

impl TaskOutcome {
    /// An outcome for a task that completed successfully by executing.
    pub fn executed(_task: &Task) -> Self {
        Self {
            status: TaskStatus::Executed,
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            duration: Duration::ZERO,
        }
    }

    /// An outcome for a task that hit the cache and didn't need to execute.
    pub fn cached(_task: &Task) -> Self {
        Self {
            status: TaskStatus::Cached,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration: Duration::ZERO,
        }
    }

    /// An outcome for a task that failed without producing output.
    pub fn failed(_task: &Task, message: impl Into<String>) -> Self {
        Self {
            status: TaskStatus::Failed,
            exit_code: None,
            stdout: String::new(),
            stderr: message.into(),
            duration: Duration::ZERO,
        }
    }
}

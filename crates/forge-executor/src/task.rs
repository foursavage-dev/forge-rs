use std::time::Duration;

use crate::command::CommandSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    pub key: String,

    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub label: String,

    pub description: String,

    pub spec: CommandSpec,

    pub cache: Option<CacheEntry>,
}

impl Task {
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

    pub fn with_cache(mut self, entry: CacheEntry) -> Self {
        self.cache = Some(entry);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Executed,

    Cached,

    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOutcome {
    pub status: TaskStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

impl TaskOutcome {
    pub fn executed(_task: &Task) -> Self {
        Self {
            status: TaskStatus::Executed,
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            duration: Duration::ZERO,
        }
    }

    pub fn cached(_task: &Task) -> Self {
        Self {
            status: TaskStatus::Cached,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration: Duration::ZERO,
        }
    }

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

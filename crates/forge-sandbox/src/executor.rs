use std::collections::HashMap;

use forge_executor::{CommandSpec, ExecutorError, Task, TaskExecutor, TaskOutcome};

use crate::env::{EnvPolicy, sanitize_env};
use crate::isolation::FsPolicy;

/// Sandboxing knobs applied to every task the executor runs.
///
/// Time limits live on `ProcessExecutor` (killing a child requires owning
/// its handle, so the process layer enforces `--timeout`); this executor
/// is purely about *what* the task sees: environment and filesystem policy.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub env_policy: EnvPolicy,
    pub fs_policy: FsPolicy,
    pub extra_env: HashMap<String, String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            env_policy: EnvPolicy::Hermetic,
            fs_policy: FsPolicy::InPlace,
            extra_env: HashMap::new(),
        }
    }
}

/// Wraps any `TaskExecutor` and rewrites the task's environment before the
/// inner executor runs it.
///
/// `EnvPolicy::Hermetic` and `EnvPolicy::Custom` mark the command
/// `env_clear` so the child sees *only* the sanitized map — otherwise
/// `CommandSpec` env entries would just be overrides on top of forge's own
/// (leaky) environment.
pub struct SandboxedExecutor<E: TaskExecutor> {
    inner: E,
    config: SandboxConfig,
}

impl<E: TaskExecutor> SandboxedExecutor<E> {
    pub fn new(inner: E, config: SandboxConfig) -> Self {
        Self { inner, config }
    }

    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    pub fn inner(&self) -> &E {
        &self.inner
    }
}

impl<E: TaskExecutor + Sync> TaskExecutor for SandboxedExecutor<E> {
    fn execute(&self, task: &Task) -> Result<TaskOutcome, ExecutorError> {
        let env = sanitize_env(&self.config.env_policy, &self.config.extra_env);

        let mut sandboxed_spec = CommandSpec::new(&task.spec.program).args(task.spec.args.clone());

        if let Some(ref cwd) = task.spec.cwd {
            sandboxed_spec = sandboxed_spec.cwd(cwd);
        }

        if self.config.env_policy != EnvPolicy::Inherit {
            sandboxed_spec = sandboxed_spec.env_clear();
        }
        for (k, v) in env {
            sandboxed_spec = sandboxed_spec.env(k, v);
        }

        let mut sandboxed_task = task.clone();
        sandboxed_task.spec = sandboxed_spec;
        self.inner.execute(&sandboxed_task)
    }
}

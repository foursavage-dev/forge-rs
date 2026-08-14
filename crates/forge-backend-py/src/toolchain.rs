use crate::config::{PyTaskSpec, PythonRunner};
use forge_executor::CommandSpec;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct PyToolchain;

impl PyToolchain {
    pub fn new() -> Self {
        Self
    }

    pub fn build_command(
        &self,
        task: &PyTaskSpec,
        runner: PythonRunner,
        root: &Path,
    ) -> CommandSpec {
        let program = task
            .command
            .clone()
            .unwrap_or_else(|| runner.executable().to_string());

        let mut spec = CommandSpec::new(program);
        for arg in &task.args {
            spec = spec.arg(arg);
        }
        spec.cwd(root)
    }
}

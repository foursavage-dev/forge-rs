use crate::config::{PexConfig, PyTaskSpec, PythonRunner};
use fish_executor::CommandSpec;
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

    pub fn build_pex_command(
        &self,
        pex_cfg: &PexConfig,
        root: &Path,
        output_path: &Path,
    ) -> CommandSpec {
        let mut spec = CommandSpec::new("pex");
        spec = spec
            .arg(".")
            .arg("-o")
            .arg(output_path.to_string_lossy().as_ref());
        if let Some(entry) = &pex_cfg.entry_point {
            spec = spec.arg("-e").arg(entry);
        }
        if let Some(ic) = &pex_cfg.interpreter_constraint {
            spec = spec.arg("--interpreter-constraint").arg(ic);
        }
        for plat in &pex_cfg.platforms {
            spec = spec.arg("--platform").arg(plat);
        }
        if let Some(inherit) = &pex_cfg.inherit_path {
            spec = spec.arg("--inherit-path").arg(inherit);
        }
        if pex_cfg.include_tools {
            spec = spec.arg("--include-tools");
        }
        spec.cwd(root)
    }
}

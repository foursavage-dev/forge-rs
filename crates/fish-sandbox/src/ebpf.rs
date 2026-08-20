use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileAccessType {
    Read,
    Write,
    Execute,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracedFileEvent {
    pub process_id: u32,
    pub path: PathBuf,
    pub access_type: FileAccessType,
}

#[derive(Debug, Clone, Default)]
pub struct EbpfTraceSummary {
    pub declared_inputs: BTreeSet<PathBuf>,
    pub undeclared_inputs: BTreeSet<PathBuf>,
    pub declared_outputs: BTreeSet<PathBuf>,
    pub undeclared_outputs: BTreeSet<PathBuf>,
}

pub struct EbpfSyscallTracer {
    enabled: bool,
    events: Vec<TracedFileEvent>,
}

impl EbpfSyscallTracer {
    pub fn new() -> Self {
        Self {
            enabled: cfg!(target_os = "linux"),
            events: Vec::new(),
        }
    }

    pub fn is_supported() -> bool {
        cfg!(target_os = "linux")
    }

    pub fn record_access(&mut self, pid: u32, path: PathBuf, access_type: FileAccessType) {
        self.events.push(TracedFileEvent {
            process_id: pid,
            path,
            access_type,
        });
    }

    pub fn analyze_hermeticity(
        &self,
        declared_inputs: &[PathBuf],
        declared_outputs: &[PathBuf],
    ) -> EbpfTraceSummary {
        let decl_inputs_set: BTreeSet<_> = declared_inputs.iter().cloned().collect();
        let decl_outputs_set: BTreeSet<_> = declared_outputs.iter().cloned().collect();

        let mut summary = EbpfTraceSummary {
            declared_inputs: decl_inputs_set.clone(),
            declared_outputs: decl_outputs_set.clone(),
            undeclared_inputs: BTreeSet::new(),
            undeclared_outputs: BTreeSet::new(),
        };

        for ev in &self.events {
            match ev.access_type {
                FileAccessType::Read => {
                    if !decl_inputs_set.contains(&ev.path) {
                        summary.undeclared_inputs.insert(ev.path.clone());
                    }
                }
                FileAccessType::Write => {
                    if !decl_outputs_set.contains(&ev.path) {
                        summary.undeclared_outputs.insert(ev.path.clone());
                    }
                }
                FileAccessType::Execute => {}
            }
        }

        summary
    }
}

impl Default for EbpfSyscallTracer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ebpf_undeclared_dependency_detection() {
        let mut tracer = EbpfSyscallTracer::new();
        let p_declared = PathBuf::from("/workspace/src/lib.rs");
        let p_undeclared = PathBuf::from("/etc/secret_config.json");
        let p_output = PathBuf::from("/workspace/target/out.o");

        tracer.record_access(1001, p_declared.clone(), FileAccessType::Read);
        tracer.record_access(1001, p_undeclared.clone(), FileAccessType::Read);
        tracer.record_access(1001, p_output.clone(), FileAccessType::Write);

        let summary = tracer.analyze_hermeticity(&[p_declared], &[p_output]);
        assert!(summary.undeclared_inputs.contains(&p_undeclared));
        assert_eq!(summary.undeclared_outputs.len(), 0);
    }
}

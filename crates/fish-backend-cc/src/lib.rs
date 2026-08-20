#![forbid(unsafe_code)]

use std::path::Path;
use thiserror::Error;

use fish_core::{BinaryUtils, BuildBackend, FingerprintUtils};
use fish_executor::{CacheEntry, CommandSpec, Task};
use fish_graph::BuildGraph;

pub mod compiler;
pub mod config;
pub mod depfile;
pub mod fingerprint;

pub use compiler::{CcCompiler, CompilerFamily};
pub use config::{CcLanguage, CcOutputType, CcProjectConfig};

#[derive(Debug, Error)]
pub enum CcBackendError {
    #[error("compiler error: {0}")]
    Compiler(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("graph error: {0}")]
    Graph(#[from] fish_graph::GraphError),
}

#[derive(Debug, Clone)]
pub struct CcBackend {
    pub compiler: CcCompiler,
}

impl BuildBackend for CcBackend {
    fn name(&self) -> &'static str {
        match self.compiler.language {
            CcLanguage::C => "c",
            CcLanguage::Cpp => "cpp",
        }
    }
}

impl CcBackend {
    pub fn new(language: CcLanguage) -> Result<Self, CcBackendError> {
        let compiler = CcCompiler::detect(language).map_err(CcBackendError::Compiler)?;
        Ok(Self { compiler })
    }

    pub fn with_compiler(compiler: CcCompiler) -> Self {
        Self { compiler }
    }

    pub fn create_tasks_from_config(
        &self,
        config: &CcProjectConfig,
        project_dir: &Path,
        output_dir: &Path,
    ) -> Result<BuildGraph<Task>, CcBackendError> {
        let mut graph = BuildGraph::new();
        let sources = config.resolve_sources(project_dir);
        let includes = config.resolve_includes(project_dir);
        std::fs::create_dir_all(output_dir.join("objs"))?;

        let flags = match self.compiler.language {
            CcLanguage::C => &config.cflags,
            CcLanguage::Cpp => &config.cxxflags,
        };

        let mut object_paths = Vec::new();
        let mut compile_node_ids = Vec::new();
        let namespace = FingerprintUtils::compute_namespace(project_dir);

        for source in &sources {
            let stem = source
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("source");

            let obj_ext =
                BinaryUtils::object_extension(self.compiler.family == CompilerFamily::Msvc);
            let obj_filename = format!("{stem}.{obj_ext}");
            let obj_path = output_dir.join("objs").join(&obj_filename);
            object_paths.push(obj_path.clone());

            let depfile = if self.compiler.family == CompilerFamily::Msvc {
                None
            } else {
                Some(output_dir.join("objs").join(format!("{stem}.d")))
            };

            let (prog, args) = self.compiler.compile_object_args(
                source,
                &obj_path,
                &includes,
                flags,
                depfile.as_deref(),
            );

            let spec = CommandSpec::new(prog).args(args).cwd(project_dir);

            let label = format!("compile {}", source.display());
            let desc = spec.command_line();

            let fingerprint_val = fingerprint::compute_source_fingerprint(
                source,
                &includes,
                flags,
                &self.compiler.version,
                depfile.as_deref(),
            )
            .unwrap_or_else(|_| "no_fp".to_string());

            let cache_entry = CacheEntry {
                key: FingerprintUtils::format_cache_key("cc", &namespace, &config.name, stem),
                fingerprint: fingerprint_val,
            };

            let task = Task::new(label, desc, spec).with_cache(cache_entry);
            let node_id = graph.add_node(task);
            compile_node_ids.push(node_id);
        }

        let out_name = BinaryUtils::add_binary_extension(&config.name);
        let final_output = output_dir.join(&out_name);

        let (link_prog, link_args) = self.compiler.link_args(
            &object_paths,
            &final_output,
            &config.ldflags,
            config.output_type,
        );

        let link_spec = CommandSpec::new(link_prog).args(link_args).cwd(project_dir);
        let link_label = format!("link {}", config.name);
        let link_desc = link_spec.command_line();

        let link_task = Task::new(link_label, link_desc, link_spec);
        let link_node_id = graph.add_node(link_task);

        for &compile_id in &compile_node_ids {
            graph.add_dependency(compile_id, link_node_id)?;
        }

        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cc_project_task_graph_construction() {
        let dummy_compiler = CcCompiler {
            executable: "gcc".to_string(),
            family: CompilerFamily::Gcc,
            version: "gcc 13.2.0".to_string(),
            language: CcLanguage::C,
        };

        let backend = CcBackend::with_compiler(dummy_compiler);

        let config = CcProjectConfig {
            name: "hello".to_string(),
            language: CcLanguage::C,
            sources: vec!["src/main.c".to_string(), "src/util.c".to_string()],
            includes: vec!["include".to_string()],
            cflags: vec!["-O2".to_string()],
            cxxflags: vec![],
            ldflags: vec!["-lm".to_string()],
            output_type: CcOutputType::Executable,
        };

        let temp = tempdir().unwrap();
        let graph = backend
            .create_tasks_from_config(&config, temp.path(), &temp.path().join("build"))
            .unwrap();

        assert_eq!(graph.len(), 3);
        assert_eq!(backend.name(), "c");

        let topo = graph.topological_order();
        assert_eq!(topo.len(), 3);

        let last_node_id = topo.last().copied().unwrap();
        let last_node = graph.node(last_node_id).unwrap();
        assert!(last_node.payload.label.starts_with("link"));
    }
}

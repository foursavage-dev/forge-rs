use std::fs;
use tempfile::tempdir;

use forge_backend_cc::{
    CcBackend, CcCompiler, CcLanguage, CcOutputType, CcProjectConfig, CompilerFamily,
};
use forge_cache::CachingExecutor;
use forge_executor::ProcessExecutor;
use forge_scheduler::Scheduler;

#[test]
fn cc_pipeline_builds_and_caches_c_project() {
    let dummy_compiler = CcCompiler {
        executable: "gcc".to_string(),
        family: CompilerFamily::Gcc,
        version: "gcc 13.2.0".to_string(),
        language: CcLanguage::C,
    };

    let backend = CcBackend::with_compiler(dummy_compiler);
    let temp = tempdir().unwrap();
    let project_dir = temp.path().join("proj");
    fs::create_dir_all(project_dir.join("src")).unwrap();

    fs::write(project_dir.join("src/main.c"), "int main() { return 0; }\n").unwrap();
    fs::write(
        project_dir.join("src/util.c"),
        "int util() { return 42; }\n",
    )
    .unwrap();

    let config = CcProjectConfig {
        name: "test_c_app".to_string(),
        language: CcLanguage::C,
        sources: vec!["src/main.c".to_string(), "src/util.c".to_string()],
        includes: vec![],
        cflags: vec!["-O2".to_string()],
        cxxflags: vec![],
        ldflags: vec![],
        output_type: CcOutputType::Executable,
    };

    let build_dir = temp.path().join("build");
    fs::create_dir_all(&build_dir).unwrap();

    let mut graph = backend
        .create_tasks_from_config(&config, &project_dir, &build_dir)
        .unwrap();

    let cache_dir = temp.path().join("cache");
    let cache = forge_cache::LocalCache::new(cache_dir).unwrap();
    let process = ProcessExecutor::new(false);
    let executor = CachingExecutor::new(process, cache);
    let scheduler = Scheduler::new(2);

    let summary = scheduler.run(&mut graph, &executor, |_, _| {}).unwrap();

    assert_eq!(summary.total, 3);
}

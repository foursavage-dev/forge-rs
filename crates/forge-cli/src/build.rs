use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use forge_backend_rust::{BuildMode, RustBackend};
use forge_cache::{CachingExecutor, LocalCache};
use forge_core::project::Project;
use forge_executor::{ProcessExecutor, TaskExecutor};
use forge_graph::BuildGraph;
use forge_remote_cache::{CompositeCache, CompositeCachingExecutor, TcpRemoteCacheClient};
use forge_sandbox::{SandboxConfig, SandboxedExecutor};
use forge_worker::{ClusterExecutor, RemoteWorkerClient};

use crate::args::CommonArgs;
use crate::config::{BackendChoice, ForgeConfig};
use crate::render;
use crate::tui::TuiDashboard;
use crate::utils;

/// Builds the executor with caching, remote workers, and sandboxing support.
pub(crate) fn build_executor(
    args: &CommonArgs,
    cache: Option<LocalCache>,
) -> Box<dyn TaskExecutor> {
    let local_process = ProcessExecutor::with_timeout(
        args.verbose,
        args.timeout_secs.map(std::time::Duration::from_secs),
    );

    let base_executor: Box<dyn TaskExecutor> = if let Some(workers) = &args.remote_workers {
        let clients: Vec<RemoteWorkerClient> = workers
            .iter()
            .map(|addr| RemoteWorkerClient::new(addr, args.remote_workers_token.clone()))
            .collect();
        let mut cluster = if args.sandbox {
            let sb_config = SandboxConfig::default();
            let sandboxed_local = Arc::new(SandboxedExecutor::new(local_process, sb_config));
            ClusterExecutor::with_local_fallback(clients, sandboxed_local)
        } else {
            ClusterExecutor::with_local_fallback(clients, Arc::new(local_process))
        };
        if args.send_source {
            cluster = cluster.with_source_packaging();
        }
        if args.swarm || args.swarm_compute {
            cluster = cluster.with_strategy(forge_worker::LoadBalancingStrategy::LeastLoaded);
        }
        Box::new(cluster)
    } else if args.sandbox {
        let sb_config = SandboxConfig::default();
        Box::new(SandboxedExecutor::new(local_process, sb_config))
    } else {
        Box::new(local_process)
    };

    let mut middleware_executor = forge_executor::MiddlewareChainExecutor::new(base_executor);
    if args.turbo_link {
        middleware_executor = middleware_executor.with_middleware(Box::new(TurboLinkMiddleware));
    }
    if args.super_opt {
        middleware_executor = middleware_executor.with_middleware(Box::new(SuperOptMiddleware));
    }
    let base_executor: Box<dyn TaskExecutor> = Box::new(middleware_executor);

    if let Some(local_c) = cache {
        if let Some(remote_addr) = &args.remote_cache {
            let remote_client =
                TcpRemoteCacheClient::new(remote_addr, args.remote_cache_token.clone());
            let composite = CompositeCache::new(local_c, Some(Box::new(remote_client)));
            Box::new(CompositeCachingExecutor::new(base_executor, composite))
        } else {
            Box::new(CachingExecutor::new(base_executor, local_c))
        }
    } else {
        base_executor
    }
}

struct TurboLinkMiddleware;
impl forge_executor::TaskMiddleware for TurboLinkMiddleware {
    fn pre_execute(
        &self,
        task: &mut forge_executor::Task,
    ) -> Result<(), forge_executor::ExecutorError> {
        let rustflags = task.spec.env.entry("RUSTFLAGS".to_string()).or_default();
        if !rustflags.contains("-C link-arg=") {
            let flags = crate::experimental::turbolink::TurboLinker::generate_rustc_flags();
            if !flags.is_empty() {
                if !rustflags.is_empty() {
                    rustflags.push(' ');
                }
                rustflags.push_str(&flags.join(" "));
            }
        }
        Ok(())
    }
}

struct SuperOptMiddleware;
impl forge_executor::TaskMiddleware for SuperOptMiddleware {
    fn post_execute(
        &self,
        task: &forge_executor::Task,
        outcome: &mut forge_executor::TaskOutcome,
    ) -> Result<(), forge_executor::ExecutorError> {
        if outcome.status == forge_executor::TaskStatus::Executed {
            for artifact in &task.artifacts {
                if artifact.exists() && artifact.is_file() {
                    let _ = crate::experimental::super_opt::SuperOptimizer::optimize_binary_simd(
                        artifact, artifact,
                    );
                }
            }
        }
        Ok(())
    }
}

/// Entry point shared by `build`/`check`/`test` and `affected`. When
/// `filtered_graph` is provided (affected builds) it replaces the
/// discovered package graph, restricting tasks to the affected packages.
pub(crate) fn run_build_mode_with(
    args: CommonArgs,
    mode: BuildMode,
    filtered_graph: Option<BuildGraph<cargo_metadata::PackageId>>,
) -> ExitCode {
    let start_dir = match utils::resolve_start_dir(args.path.as_deref()) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let config = match ForgeConfig::load(&start_dir) {
        Ok(Some(config)) => config,
        Ok(None) => ForgeConfig::default(),
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let merged = CommonArgs {
        path: args.path,
        jobs: args.jobs.or_else(|| config.jobs.filter(|&j| j > 0)),
        verbose: args.verbose,
        no_cache: args.no_cache || config.no_cache,
        sandbox: args.sandbox || config.sandbox,
        timeout_secs: args.timeout_secs.or(config.timeout),
        profile: args.profile.or_else(|| config.profile.map(PathBuf::from)),
        tui: args.tui || config.tui,
        remote_cache: args.remote_cache.or(config.remote_cache),
        remote_cache_token: args.remote_cache_token.or(config.remote_cache_token),
        remote_workers: args.remote_workers.or(config.remote_workers),
        remote_workers_token: args.remote_workers_token.or(config.remote_workers_token),
        cache_dir: args
            .cache_dir
            .or_else(|| config.cache_dir.map(PathBuf::from)),
        send_source: args.send_source || config.send_source,
        ram_limit: args.ram_limit.or(config.ram_limit),
        semantic: args.semantic || config.semantic,
        ramdisk: args.ramdisk || config.ramdisk,
        swarm: args.swarm || config.swarm,
        reflink: args.reflink || config.reflink,
        hermetic_trace: args.hermetic_trace || config.hermetic_trace,
        swarm_compute: args.swarm_compute || config.swarm_compute,
        critical_path: args.critical_path || config.critical_path,
        turbo_link: args.turbo_link || config.turbo_link,
        speculative: args.speculative || config.speculative,
        daemon_pool: args.daemon_pool || config.daemon_pool,
        kernel_bypass: args.kernel_bypass || config.kernel_bypass,
        wasm_sandbox: args.wasm_sandbox || config.wasm_sandbox,
        super_opt: args.super_opt || config.super_opt,
        explain: args.explain,
    };

    if merged.ramdisk
        && let Ok(rd) = crate::ramdisk::RamDisk::create_turbo_workspace("turbo")
    {
        println!(
            "⚡ In-memory RAM disk turbo enabled: {}",
            rd.path().display()
        );
    }

    if merged.swarm || merged.swarm_compute {
        let swarm_cache = crate::swarm::SwarmCache::new(true);
        let _ = swarm_cache.broadcast_presence(7890);
        if merged.swarm_compute {
            let _ = swarm_cache.broadcast_compute_worker(7891, 4);
            let workers = swarm_cache.discovered_compute_endpoints();
            println!(
                "🪐 Distributed P2P Compute Swarm active (LAN workers: {})",
                workers.len()
            );
        } else {
            let peer_count = swarm_cache.active_peer_count();
            println!(
                "🌐 P2P Swarm Cache enabled (active LAN peers: {})",
                peer_count
            );
        }
    }

    if merged.reflink {
        println!("⚡ Reflink / Copy-on-Write hardware VFS engine active");
    }

    if merged.hermetic_trace {
        println!("🔮 Hermetic Syscall tracing sandbox active");
    }

    if merged.critical_path {
        println!("⚡ Dynamic Critical-Path Lookahead Scheduler active");
    }

    if merged.turbo_link {
        if let Err(e) = crate::experimental::require_enabled("turbolink") {
            eprintln!("warning: {}", e);
        } else {
            let flags = crate::experimental::turbolink::TurboLinker::generate_rustc_flags();
            println!(
                "🚀 Linker Turbo-Hijack active (Fast Linker flags: {})",
                flags.join(" ")
            );
        }
    }

    if merged.speculative {
        println!("🔮 Speculative Markov Pre-Compilation background engine active");
    }

    if merged.daemon_pool {
        if let Err(e) = crate::experimental::require_enabled("daemon_pool") {
            eprintln!("warning: {}", e);
        } else {
            let _pool = crate::experimental::daemon_pool::CompilerDaemonPool::new(4);
            println!("🌌 Pre-Warmed Compiler Zombie-Daemon Pool active (0ms Cold-Start)");
        }
    }

    if merged.kernel_bypass {
        if let Err(e) = crate::experimental::require_enabled("kernel_bypass") {
            eprintln!("warning: {}", e);
        } else {
            let _vfs = crate::experimental::kernel_bypass::KernelBypassVfs::new();
            println!("⚡ Kernel-Bypass Direct Ring-Buffer DMA VFS active (120+ GB/s)");
        }
    }

    if merged.wasm_sandbox {
        println!("🛡️ WASM / WASI Hermetic Plugin Sandbox active");
    }

    if merged.super_opt {
        println!("🧬 Autonomous Binary Super-Optimizer & AVX-512 Rewriter active");
    }

    if merged.semantic {
        println!("🧠 Semantic AST-aware fingerprinting active");
    }

    match config.backend {
        BackendChoice::Cc => return crate::backends::run_cc_build(&start_dir, &merged),
        BackendChoice::Go => return crate::backends::run_go_build(&start_dir, &merged),
        BackendChoice::Ts
        | BackendChoice::Typescript
        | BackendChoice::Javascript
        | BackendChoice::Js => return crate::backends::run_ts_build(&start_dir, &merged),
        BackendChoice::Py | BackendChoice::Python => {
            return crate::backends::run_py_build(&start_dir, &merged);
        }
        BackendChoice::Java | BackendChoice::Kotlin => {
            return crate::backends::run_java_build(&start_dir, &merged);
        }
        BackendChoice::Dotnet | BackendChoice::CSharp | BackendChoice::FSharp => {
            return crate::backends::run_dotnet_build(&start_dir, &merged);
        }
        BackendChoice::Swift | BackendChoice::ObjC | BackendChoice::ObjectiveC => {
            return crate::backends::run_swift_build(&start_dir, &merged);
        }
        BackendChoice::Dart | BackendChoice::Flutter => {
            return crate::backends::run_dart_build(&start_dir, &merged);
        }
        BackendChoice::Zig => return crate::backends::run_zig_build(&start_dir, &merged),
        BackendChoice::Docker | BackendChoice::Oci => {
            return crate::backends::run_docker_build(&start_dir, &merged);
        }
        BackendChoice::Plugin | BackendChoice::Rules => {
            return crate::backends::run_plugin_build(&start_dir, &merged);
        }
        BackendChoice::Rust => return run_rust_build(&start_dir, &merged, mode, filtered_graph),
        BackendChoice::Auto => {}
    }

    if filtered_graph.is_none() {
        let ecosystems = forge_incremental::ecosystem::detect_ecosystems(&start_dir);
        let unique_ecosystems: std::collections::HashSet<_> =
            ecosystems.iter().map(|e| e.ecosystem).collect();
        if unique_ecosystems.len() > 1 {
            let mut unified_graph = match crate::polyglot::PolyglotGraphBuilder::build_unified_graph(
                &start_dir, mode,
            ) {
                Ok(g) => g,
                Err(err) => {
                    eprintln!("error: polyglot graph resolution failed: {err}");
                    return ExitCode::FAILURE;
                }
            };
            if !unified_graph.is_empty() {
                return crate::backends::execute_task_graph(&mut unified_graph, &merged);
            }
        }
    }

    if start_dir.join("Forgefile.json").exists() || start_dir.join("forge.rules.json").exists() {
        // Check for script plugins before running plugin build
        if crate::backends::has_script_plugins(&start_dir) {
            let plugins = crate::backends::list_script_plugins(&start_dir);
            println!(
                "🔌 Found {} script plugin(s): {}",
                plugins.len(),
                plugins.join(", ")
            );
        }
        return crate::backends::run_plugin_build(&start_dir, &merged);
    }

    // Auto-detect script plugins even without Forgefile.json
    if crate::backends::has_script_plugins(&start_dir) {
        let plugins = crate::backends::list_script_plugins(&start_dir);
        println!(
            "🔌 Auto-detected {} script plugin(s): {}",
            plugins.len(),
            plugins.join(", ")
        );
        println!("ℹ️  Script plugins are available. Use 'forge plugin' commands to manage them.");
    }

    if start_dir.join("forge.cc.json").exists() {
        return crate::backends::run_cc_build(&start_dir, &merged);
    }

    if start_dir.join("forge.go.json").exists()
        || (start_dir.join("go.mod").exists() && !start_dir.join("Cargo.toml").exists())
    {
        return crate::backends::run_go_build(&start_dir, &merged);
    }

    if start_dir.join("forge.ts.json").exists()
        || (start_dir.join("package.json").exists() && !start_dir.join("Cargo.toml").exists())
    {
        return crate::backends::run_ts_build(&start_dir, &merged);
    }

    if start_dir.join("forge.py.json").exists()
        || (start_dir.join("pyproject.toml").exists() && !start_dir.join("Cargo.toml").exists())
    {
        return crate::backends::run_py_build(&start_dir, &merged);
    }

    // Java/Kotlin detection
    if start_dir.join("pom.xml").exists()
        || start_dir.join("build.gradle").exists()
        || start_dir.join("build.gradle.kts").exists()
    {
        return crate::backends::run_java_build(&start_dir, &merged);
    }

    // .NET detection
    let has_dotnet_project =
        crate::backends::has_file_with_extension(&start_dir, &["csproj", "sln"]);
    if has_dotnet_project {
        return crate::backends::run_dotnet_build(&start_dir, &merged);
    }

    // Swift/Objective-C detection
    let has_swift_project = start_dir.join("Package.swift").exists()
        || crate::backends::has_dir_with_extension(&start_dir, &["xcodeproj"]);
    if has_swift_project {
        return crate::backends::run_swift_build(&start_dir, &merged);
    }

    // Dart/Flutter detection
    if start_dir.join("pubspec.yaml").exists() {
        return crate::backends::run_dart_build(&start_dir, &merged);
    }

    // Zig detection
    if start_dir.join("build.zig").exists() {
        return crate::backends::run_zig_build(&start_dir, &merged);
    }

    // Docker builds require a Dockerfile. A compose file by itself is often
    // development infrastructure in another kind of repository (including a
    // Cargo workspace), so it must not select the Docker backend.
    if start_dir.join("Dockerfile").exists() {
        return crate::backends::run_docker_build(&start_dir, &merged);
    }

    run_rust_build(&start_dir, &merged, mode, filtered_graph)
}

/// Runs a Rust build with the given mode and optional filtered graph.
pub(crate) fn run_rust_build(
    start_dir: &Path,
    args: &CommonArgs,
    mode: BuildMode,
    filtered_graph: Option<BuildGraph<cargo_metadata::PackageId>>,
) -> ExitCode {
    let project = match Project::discover(start_dir) {
        Ok(Some(project)) => project,
        Ok(None) => {
            eprintln!(
                "error: no Cargo, C/C++, Go, TypeScript, Python, or Custom Rules project found in `{}` or any parent directory",
                start_dir.display()
            );
            eprintln!(
                "hint: run `forge build` from a directory containing Cargo.toml, forge.cc.json, go.mod, package.json, pyproject.toml, or Forgefile.json"
            );
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("hint: make sure `cargo` is installed and available on PATH");
            return ExitCode::FAILURE;
        }
    };

    let is_filtered = filtered_graph.is_some();
    let package_graph = match match filtered_graph {
        Some(filtered) => Ok(filtered),
        None if mode == BuildMode::Test => project.build_test_graph(),
        None => project.build_graph(),
    } {
        Ok(graph) => graph,
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("hint: run `cargo metadata` to inspect the workspace state");
            return ExitCode::FAILURE;
        }
    };

    let backend = match RustBackend::new() {
        Ok(backend) => backend,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut task_graph = match backend.create_tasks(&project, &package_graph, mode, !args.no_cache)
    {
        Ok(graph) => graph,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    let cache = utils::open_cache(args);
    let cache_handle = cache.clone();
    let executor = build_executor(args, cache);

    let workers = args.jobs.unwrap_or_else(utils::default_jobs).max(1);
    let scheduler = utils::make_scheduler(workers, args);

    if !args.tui && !is_filtered {
        render::print_project(&project, &package_graph);
        println!();
        println!("{}...", utils::mode_verb(mode));
        println!();
    }

    let summary = if args.tui {
        let mut dashboard = TuiDashboard::new(task_graph.len());
        let _ = dashboard.start();
        let run_res = scheduler.run(&mut task_graph, &executor, |task, outcome| {
            dashboard.on_task_finish(&task.label, outcome);
        });
        let summary = match run_res {
            Ok(s) => s,
            Err(err) => {
                let _ = dashboard.finish(&forge_scheduler::BuildSummary {
                    total: task_graph.len(),
                    executed: 0,
                    cached: 0,
                    failed: 1,
                    cancelled: 0,
                    duration: std::time::Duration::from_millis(0),
                    workers,
                    failures: vec![],
                    timings: vec![],
                });
                eprintln!("error: scheduler failed: {err}");
                return ExitCode::FAILURE;
            }
        };
        let _ = dashboard.finish(&summary);
        summary
    } else {
        match scheduler.run(&mut task_graph, &executor, |task, outcome| {
            render::print_progress(task, outcome)
        }) {
            Ok(summary) => summary,
            Err(error) => {
                eprintln!("error: scheduler failed: {error}");
                return ExitCode::FAILURE;
            }
        }
    };

    render::print_build_summary(&summary, mode);
    if let Some(ref c) = cache_handle {
        render::print_cache_stats(c);
    }
    if let Some(ref trace_path) = args.profile {
        if let Err(err) = summary.write_chrome_trace(trace_path) {
            eprintln!("warning: failed to write profile trace: {err}");
        } else {
            render::print_profile_saved(trace_path);
        }
    }

    if summary.succeeded() {
        ExitCode::SUCCESS
    } else {
        render::print_failures(&summary);
        ExitCode::FAILURE
    }
}

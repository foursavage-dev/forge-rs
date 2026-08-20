use std::fmt;
use std::path::Path;

use anstream::println;
use anstyle::{AnsiColor, Color, Effects, Style};

use cargo_metadata::PackageId;
use fish_backend_rust::BuildMode;
use fish_core::project::Project;
use fish_graph::BuildGraph;

const BOLD: Style = Style::new().effects(Effects::BOLD);
const DIM: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));
const GREEN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
const RED: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));

fn bullet(prefix: &str, text: &str) -> String {
    format!("{prefix} {text}")
}

struct Styled {
    style: Style,
    text: String,
}

impl Styled {
    fn new(style: Style, text: impl Into<String>) -> Self {
        Self {
            style,
            text: text.into(),
        }
    }
}

impl fmt::Display for Styled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.style.render(),
            self.text,
            self.style.render_reset()
        )
    }
}

pub fn print_project(project: &Project, graph: &BuildGraph<PackageId>) {
    println!(
        "{}",
        Styled::new(BOLD, format!("Fish 🦀 {}", env!("CARGO_PKG_VERSION")))
    );
    println!();

    let metadata = project.metadata();
    let project_name = project
        .root_package()
        .map(|package| package.name.to_string())
        .unwrap_or_else(|| "(workspace)".to_string());

    println!("Project:              {project_name}");
    println!(
        "Manifest:             {}",
        project.manifest_path().display()
    );
    println!("Workspace:            {}", workspace_label(project));
    println!(
        "Workspace packages:   {} ({} default)",
        metadata.workspace_members.len(),
        metadata.workspace_default_members.len()
    );

    if !graph.is_empty() {
        println!();
        println!("{}", Styled::new(DIM, "Build graph:"));
        print_levels(project, graph);
    }
}

fn package_name(project: &Project, payload: &PackageId) -> String {
    project
        .package(payload)
        .map(|package| package.name.to_string())
        .unwrap_or_else(|| payload.to_string())
}

fn print_levels(project: &Project, graph: &BuildGraph<PackageId>) {
    let levels = graph.levels();
    for (level_index, level) in levels.iter().enumerate() {
        let count = level.len();
        let stage_label = format!(
            "Stage {} ({} package{})",
            level_index + 1,
            count,
            if count == 1 { "" } else { "s" }
        );
        println!("  {}", Styled::new(DIM, &stage_label));
        for (i, id) in level.iter().enumerate() {
            let is_last = i + 1 == level.len();
            let branch = if is_last {
                "  └── "
            } else {
                "  ├── "
            };
            let name = graph
                .node(*id)
                .map(|node| package_name(project, &node.payload))
                .unwrap_or_else(|| format!("{id:?}"));
            println!("{branch}{name}");
        }
        if level_index + 1 < levels.len() {
            println!("  │");
            println!("  ▼");
        }
    }
}

fn workspace_label(project: &Project) -> String {
    let root = project.workspace_root().as_str();
    let name = project.workspace_root().file_name().unwrap_or(root);
    if project.is_workspace() {
        format!("{name} (workspace)")
    } else {
        format!("{name} (single package)")
    }
}

pub fn print_progress(task: &fish_executor::Task, outcome: &fish_executor::TaskOutcome) {
    match outcome.status {
        fish_executor::TaskStatus::Executed => {
            println!("{}", Styled::new(GREEN, bullet("✓", &task.label)))
        }
        fish_executor::TaskStatus::Cached => println!(
            "{}",
            Styled::new(DIM, bullet("✓", &format!("{} (cached)", task.label)))
        ),
        fish_executor::TaskStatus::Failed => {
            println!("{}", Styled::new(RED, bullet("✗", &task.label)))
        }
    }
}

pub fn print_build_summary(summary: &fish_scheduler::BuildSummary, mode: BuildMode) {
    if summary.succeeded() {
        let message = match mode {
            BuildMode::Build => "Build completed successfully.",
            BuildMode::Check => "Check completed successfully.",
            BuildMode::Test => "All tests passed.",
            BuildMode::Clippy => "Clippy checks passed.",
            BuildMode::Doc => "Documentation generated.",
            BuildMode::Bench => "Benchmarks finished.",
        };
        println!("{}", Styled::new(GREEN, message));
    } else {
        let message = match mode {
            BuildMode::Build => "Build failed.",
            BuildMode::Check => "Check failed.",
            BuildMode::Test => "Some tests failed.",
            BuildMode::Clippy => "Clippy checks failed.",
            BuildMode::Doc => "Documentation build failed.",
            BuildMode::Bench => "Benchmarks failed.",
        };
        println!("{}", Styled::new(RED, message));
    }
    println!("  Tasks:     {} total", summary.total);
    println!("  Executed:  {}", summary.executed);
    println!("  Cached:    {}", summary.cached);
    println!("  Failed:    {}", summary.failed);
    if summary.cancelled > 0 {
        println!("  Cancelled: {}", summary.cancelled);
    }
    println!("  Workers:   {}", summary.workers);
    println!("  Duration:  {:.2}s", summary.duration.as_secs_f64());
}

pub fn print_cache_location(root: &Path) {
    println!("Cache:                {}", root.display());
}

pub fn print_cache_stats(cache: &fish_cache::LocalCache) {
    let stats = cache.stats();
    println!(
        "  Cache:     {} hits, {} misses, {} errors",
        stats.hits(),
        stats.misses(),
        stats.errors()
    );
}

pub fn print_profile_saved(path: &Path) {
    println!("  Trace:     {} (open in ui.perfetto.dev)", path.display());
}

pub fn print_failures(summary: &fish_scheduler::BuildSummary) {
    for failure in &summary.failures {
        eprintln!();
        eprintln!("Task:      {}", failure.label);
        eprintln!("Command:   {}", failure.description);
        eprintln!("Output:");
        let mut output = String::new();
        if !failure.stdout.trim().is_empty() {
            output.push_str(&failure.stdout);
        }
        if !failure.stderr.trim().is_empty() {
            output.push_str(&failure.stderr);
        }
        let total = output.lines().count();
        for (index, line) in output.lines().enumerate() {
            if index == 30 {
                eprintln!("  … ({} more lines)", total - 30);
                break;
            }
            eprintln!("  {line}");
        }
        if output.trim().is_empty() {
            eprintln!("  (no output)");
        }
    }
}

pub fn print_graph_tree(project: &Project, graph: &BuildGraph<PackageId>) {
    let mut roots: Vec<fish_graph::NodeId> = Vec::new();
    for node in graph.nodes() {
        if graph.dependents(node.id).unwrap_or(&[]).is_empty() {
            roots.push(node.id);
        }
    }

    roots.sort_by_key(|&id| {
        graph
            .node(id)
            .map(|n| package_name(project, &n.payload))
            .unwrap_or_default()
    });

    for (i, &root) in roots.iter().enumerate() {
        print_tree_node(project, graph, root, "", i == roots.len() - 1, true);
    }
}

fn print_tree_node(
    project: &Project,
    graph: &BuildGraph<PackageId>,
    node_id: fish_graph::NodeId,
    prefix: &str,
    is_last: bool,
    is_root: bool,
) {
    let node = graph.node(node_id).unwrap();
    let name = package_name(project, &node.payload);

    let connector = if is_root {
        ""
    } else if is_last {
        "└── "
    } else {
        "├── "
    };

    println!("{}{}{}", prefix, connector, name);

    let deps_slice = graph.deps(node_id).unwrap_or_default();
    let mut deps_vec: Vec<fish_graph::NodeId> = deps_slice.to_vec();
    deps_vec.sort_by_key(|&id| {
        graph
            .node(id)
            .map(|n| package_name(project, &n.payload))
            .unwrap_or_default()
    });

    let child_prefix = if is_root {
        "".to_string()
    } else if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    for (i, dep) in deps_vec.iter().enumerate() {
        print_tree_node(
            project,
            graph,
            *dep,
            &child_prefix,
            i == deps_vec.len() - 1,
            false,
        );
    }
}

pub fn print_graph_json(project: &Project, graph: &BuildGraph<PackageId>) {
    let mut nodes_json = vec![];
    for node in graph.nodes() {
        let node_id = node.id;
        let name = package_name(project, &node.payload);

        let mut deps_names = vec![];
        for &dep in graph.deps(node_id).unwrap_or(&[]) {
            if let Some(dep_node) = graph.node(dep) {
                deps_names.push(package_name(project, &dep_node.payload));
            }
        }
        deps_names.sort();

        nodes_json.push(serde_json::json!({
            "name": name,
            "deps": deps_names,
        }));
    }
    nodes_json.sort_by_key(|v| v["name"].as_str().unwrap_or_default().to_string());

    let mut levels_json = Vec::new();
    for level in graph.levels() {
        let mut level_names: Vec<_> = level
            .iter()
            .filter_map(|&id| graph.node(id).map(|n| package_name(project, &n.payload)))
            .collect();
        level_names.sort();
        levels_json.push(level_names);
    }

    let output = serde_json::json!({
        "nodes": nodes_json,
        "levels": levels_json,
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

pub fn print_graph_dot(project: &Project, graph: &BuildGraph<PackageId>) {
    println!("digraph forge {{");
    println!("    rankdir=BT;");

    let mut edges = vec![];
    for node in graph.nodes() {
        let node_id = node.id;
        let name = package_name(project, &node.payload);
        for &dep in graph.deps(node_id).unwrap_or(&[]) {
            if let Some(dep_node) = graph.node(dep) {
                let dep_name = package_name(project, &dep_node.payload);
                edges.push((dep_name, name.clone()));
            }
        }
    }

    edges.sort();
    for (from, to) in edges {
        println!("    \"{}\" -> \"{}\";", from, to);
    }

    println!("}}");
}

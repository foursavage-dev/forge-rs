//! Terminal rendering for the Forge CLI.

use std::fmt;
use std::path::Path;

use anstream::println;
use anstyle::{AnsiColor, Color, Effects, Style};

use cargo_metadata::PackageId;
use forge_core::project::Project;
use forge_graph::BuildGraph;

const BOLD: Style = Style::new().effects(Effects::BOLD);
const DIM: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));
const GREEN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
const RED: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));

/// Emoji bullet with `prefix`, like "✓ label" or "✓ label (cached)".
fn bullet(prefix: &str, text: &str) -> String {
    format!("{prefix} {text}")
}

/// Text wrapped in ANSI styling, rendered only as escape sequences.
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

/// Print the project summary and the workspace build graph.
pub fn print_project(project: &Project, graph: &BuildGraph<PackageId>) {
    println!(
        "{}",
        Styled::new(BOLD, format!("Forge 🦀 {}", env!("CARGO_PKG_VERSION")))
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

/// Render the graph as horizontal levels; each arrow column belongs to the
/// node above it.
fn print_levels(project: &Project, graph: &BuildGraph<PackageId>) {
    const INDENT: usize = 4;
    let levels = graph.levels();
    let cell_width = levels
        .iter()
        .flatten()
        .map(|id| {
            graph
                .node(*id)
                .map(|node| package_name(project, &node.payload).chars().count())
                .unwrap_or(0)
        })
        .max()
        .unwrap_or(0)
        + 2;

    for (level_index, level) in levels.iter().enumerate() {
        let row: String = level
            .iter()
            .map(|id| {
                let name = graph
                    .node(*id)
                    .map(|node| package_name(project, &node.payload))
                    .unwrap_or_else(|| format!("{id:?}"));
                format!("{:>width$}", name, width = INDENT + cell_width)
            })
            .collect();
        println!("{row}");

        if level_index + 1 < levels.len() {
            let arrows: String = level
                .iter()
                .map(|id| {
                    let name_len = graph
                        .node(*id)
                        .map(|node| package_name(project, &node.payload).chars().count())
                        .unwrap_or(0);
                    format!(
                        "{:>width$}",
                        "↓",
                        width = INDENT + cell_width - (name_len - 1) / 2
                    )
                })
                .collect();
            println!("{arrows}");
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

/// One line of live progress: tick + label, dimmed when cached.
pub fn print_progress(task: &forge_executor::Task, outcome: &forge_executor::TaskOutcome) {
    match outcome.status {
        forge_executor::TaskStatus::Executed => {
            println!("{}", Styled::new(GREEN, bullet("✓", &task.label)))
        }
        forge_executor::TaskStatus::Cached => println!(
            "{}",
            Styled::new(DIM, bullet("✓", &format!("{} (cached)", task.label)))
        ),
        forge_executor::TaskStatus::Failed => {
            println!("{}", Styled::new(RED, bullet("✗", &task.label)))
        }
    }
}

/// The post-build summary block.
pub fn print_build_summary(summary: &forge_scheduler::BuildSummary) {
    if summary.succeeded() {
        println!("{}", Styled::new(GREEN, "Build completed successfully."));
    } else {
        println!("{}", Styled::new(RED, "Build failed."));
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

/// Where the fingerprint cache lives (printed once at the start of a build).
pub fn print_cache_location(root: &Path) {
    println!("Cache:                {}", root.display());
}

/// Cache hit/miss/error counters, printed after the build summary.
pub fn print_cache_stats(cache: &forge_cache::LocalCache) {
    let stats = cache.stats();
    println!(
        "  Cache:     {} hits, {} misses, {} errors",
        stats.hits(),
        stats.misses(),
        stats.errors()
    );
}

/// Details of every failed task, with a tail of its stderr.
pub fn print_failures(summary: &forge_scheduler::BuildSummary) {
    for failure in &summary.failures {
        eprintln!();
        eprintln!("Task:      {}", failure.label);
        eprintln!("Command:   {}", failure.description);
        eprintln!("Output:");
        for (index, line) in failure.stderr.lines().enumerate() {
            if index == 30 {
                eprintln!("  … ({} more lines)", failure.stderr.lines().count() - 30);
                break;
            }
            eprintln!("  {line}");
        }
        if failure.stderr.trim().is_empty() {
            eprintln!("  (no output)");
        }
    }
}

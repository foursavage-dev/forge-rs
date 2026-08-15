use std::process::ExitCode;

use forge_core::project::Project;

use crate::args::{GraphArgs, GraphFormat};
use crate::utils::resolve_start_dir;
use crate::render;

pub fn run_graph(args: GraphArgs) -> ExitCode {
    let start_dir = match resolve_start_dir(args.path.as_deref()) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let project = match Project::discover(&start_dir) {
        Ok(Some(project)) => project,
        Ok(None) => {
            eprintln!(
                "error: no Cargo project found in `{}` or any parent directory",
                start_dir.display()
            );
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    let graph = match project.build_graph() {
        Ok(graph) => graph,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    match args.format {
        GraphFormat::Tree => render::print_graph_tree(&project, &graph),
        GraphFormat::Json => render::print_graph_json(&project, &graph),
        GraphFormat::Dot => render::print_graph_dot(&project, &graph),
    }

    ExitCode::SUCCESS
}

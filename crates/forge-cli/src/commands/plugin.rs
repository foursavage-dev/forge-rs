use std::process::ExitCode;

use forge_plugin::scripting::PluginError;

use crate::args::PluginArgs;
use crate::backends;

pub fn run_plugin(args: PluginArgs) -> ExitCode {
    let start_dir = match crate::utils::resolve_start_dir(args.path.as_deref()) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    match args.action {
        crate::args::PluginAction::List => {
            let plugins = backends::list_script_plugins(&start_dir);
            if plugins.is_empty() {
                println!("No script plugins found in .forge/plugins/");
            } else {
                println!("Available script plugins:");
                for plugin in &plugins {
                    println!("  - {}", plugin);
                }
            }
            ExitCode::SUCCESS
        }
        crate::args::PluginAction::Execute { name, command, args: plugin_args } => {
            match backends::execute_script_plugin(&start_dir, &name, &command, &plugin_args) {
                Ok(output) => {
                    if !output.stdout.is_empty() {
                        print!("{}", output.stdout);
                    }
                    if !output.stderr.is_empty() {
                        eprint!("{}", output.stderr);
                    }
                    if output.success {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::FAILURE
                    }
                }
                Err(PluginError::InvalidConfig(msg)) => {
                    eprintln!("error: {}", msg);
                    ExitCode::FAILURE
                }
                Err(PluginError::Execution { command: cmd, message }) => {
                    eprintln!("error: plugin '{}' failed: {}", cmd, message);
                    ExitCode::FAILURE
                }
                Err(PluginError::Unsupported(msg)) => {
                    eprintln!("error: {}", msg);
                    ExitCode::FAILURE
                }
                Err(PluginError::DependencyMissing(dep)) => {
                    eprintln!("error: missing plugin dependency: {}", dep);
                    ExitCode::FAILURE
                }
            }
        }
    }
}

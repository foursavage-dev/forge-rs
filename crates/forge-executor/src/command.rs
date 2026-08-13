//! A declarative description of a process to run: `CommandSpec`.
//!
//! `CommandSpec` is the recipe a task's executor turns into an OS process.
//! It stays serializable and pure (no side effects on construction), so a
//! task graph can be built, inspected, and executed freely.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

/// A fully-specified external command: program, arguments, environment, cwd.
///
/// Forge never shells out to a command interpreter; the `program` is
/// resolved and spawned directly (see `std::process::Command`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    /// The program to execute.
    pub program: String,
    /// Positional arguments, in order.
    pub args: Vec<String>,
    /// Environment overrides applied on top of the inherited environment.
    pub env: BTreeMap<String, String>,
    /// Working directory for the child process.
    pub cwd: Option<PathBuf>,
}

impl CommandSpec {
    /// Create a command with no arguments.
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: None,
        }
    }

    /// Append an argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Append several arguments.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Set an environment variable override.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set the working directory.
    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Render the command as a single shell-like string (for display only).
    ///
    /// `program` is rendered verbatim; arguments are single-quoted if they
    /// contain whitespace or shell metacharacters.
    pub fn command_line(&self) -> String {
        let mut line = String::new();
        write!(&mut line, "{}", self.program).unwrap();
        for arg in &self.args {
            if arg.chars().any(char::is_whitespace)
                || arg.chars().any(|c| "\"$`\\;|&<>()[]{}*?!~".contains(c))
            {
                write!(&mut line, " '{}'", arg).unwrap();
            } else {
                write!(&mut line, " {arg}").unwrap();
            }
        }
        line
    }

    /// Convert into a `std::process::Command`, ready to run.
    pub fn to_std_command(&self) -> std::process::Command {
        let mut command = std::process::Command::new(&self.program);
        command.args(&self.args);
        for (key, value) in &self.env {
            command.env(key, value);
        }
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }
        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_line_quotes_arguments_with_spaces() {
        let spec = CommandSpec::new("cargo")
            .arg("build")
            .arg("--manifest-path")
            .arg("a path/to/Cargo.toml");
        assert_eq!(
            spec.command_line(),
            "cargo build --manifest-path 'a path/to/Cargo.toml'"
        );
    }

    #[test]
    fn command_line_leaves_plain_arguments_alone() {
        let spec = CommandSpec::new("cargo").arg("build").arg("--release");
        assert_eq!(spec.command_line(), "cargo build --release");
    }

    #[test]
    fn std_command_roundtrip() {
        let spec = CommandSpec::new("cargo")
            .arg("--version")
            .env("FORGE_TEST", "1")
            .cwd(std::env::temp_dir());
        let std_command = spec.to_std_command();
        assert_eq!(std_command.get_program(), "cargo");
        let args: Vec<&std::ffi::OsStr> = std_command.get_args().collect();
        assert_eq!(args, vec!["--version"]);
        assert_eq!(std_command.get_envs().count(), 1);
    }
}

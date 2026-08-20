use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,

    pub args: Vec<String>,

    pub env: BTreeMap<String, String>,

    pub env_clear: bool,

    pub cwd: Option<PathBuf>,
}

impl CommandSpec {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
            env_clear: false,
            cwd: None,
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn env_clear(mut self) -> Self {
        self.env_clear = true;
        self
    }

    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

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

    pub fn to_std_command(&self) -> std::process::Command {
        let mut command = std::process::Command::new(&self.program);
        command.args(&self.args);
        if self.env_clear {
            command.env_clear();
        }
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
            .env("FISH_TEST", "1")
            .cwd(std::env::temp_dir());
        let std_command = spec.to_std_command();
        assert_eq!(std_command.get_program(), "cargo");
        let args: Vec<&std::ffi::OsStr> = std_command.get_args().collect();
        assert_eq!(args, vec!["--version"]);
        assert_eq!(std_command.get_envs().count(), 1);
    }

    #[test]
    fn env_clear_replaces_the_inherited_environment() {
        let spec = CommandSpec::new("cmd").env_clear().env("FISH_ONLY", "1");
        let std_command = spec.to_std_command();
        let envs: Vec<_> = std_command.get_envs().collect();
        assert_eq!(
            envs,
            vec![(
                std::ffi::OsStr::new("FISH_ONLY"),
                Some(std::ffi::OsStr::new("1"))
            )]
        );
    }
}

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileCommand {
    pub directory: String,
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilationDatabase {
    pub commands: Vec<CompileCommand>,
}

impl CompilationDatabase {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn add_command(&mut self, cmd: CompileCommand) {
        self.commands.push(cmd);
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).with_context(|| {
            format!(
                "Failed to read compilation database from {:?}",
                path.as_ref()
            )
        })?;
        let commands: Vec<CompileCommand> = serde_json::from_str(&content)
            .with_context(|| "Failed to parse compilation database JSON")?;
        Ok(Self { commands })
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.commands)
            .context("Failed to serialize compilation database to JSON")
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = self.to_json()?;
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create parent directory for {:?}", path.as_ref())
            })?;
        }
        std::fs::write(path.as_ref(), json).with_context(|| {
            format!(
                "Failed to write compilation database to {:?}",
                path.as_ref()
            )
        })?;
        Ok(())
    }

    pub fn generate_for_workspace<P: AsRef<Path>>(
        workspace_root: P,
        packages: &[(String, String, Vec<String>)],
    ) -> Self {
        let mut db = Self::new();
        let root_str = workspace_root.as_ref().to_string_lossy().to_string();

        for (pkg_name, main_file, flags) in packages {
            let mut args = vec![pkg_name.clone()];
            args.extend(flags.iter().cloned());
            args.push(main_file.clone());

            let cmd_str = args.join(" ");

            db.add_command(CompileCommand {
                directory: root_str.clone(),
                file: main_file.clone(),
                command: Some(cmd_str),
                arguments: Some(args),
                output: Some(format!("target/debug/build/{}", pkg_name)),
            });
        }

        db
    }

    pub fn count(&self) -> usize {
        self.commands.len()
    }

    pub fn find_for_file<P: AsRef<Path>>(&self, file_path: P) -> Option<&CompileCommand> {
        let target = file_path.as_ref().to_string_lossy();
        self.commands
            .iter()
            .find(|cmd| cmd.file == target.as_ref() || cmd.file.ends_with(target.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation_database_serialization() {
        let mut db = CompilationDatabase::new();
        db.add_command(CompileCommand {
            directory: "/workspace".to_string(),
            file: "src/main.rs".to_string(),
            command: Some("rustc src/main.rs".to_string()),
            arguments: Some(vec!["rustc".to_string(), "src/main.rs".to_string()]),
            output: Some("target/debug/main".to_string()),
        });

        assert_eq!(db.count(), 1);
        let json = db.to_json().unwrap();
        assert!(json.contains("src/main.rs"));

        let found = db.find_for_file("src/main.rs");
        assert!(found.is_some());
        assert_eq!(found.unwrap().directory, "/workspace");
    }

    #[test]
    fn test_generate_and_write_file() {
        let temp = tempfile::tempdir().unwrap();
        let out_file = temp.path().join("compile_commands.json");

        let pkgs = vec![
            (
                "app".to_string(),
                "src/main.rs".to_string(),
                vec!["--edition=2021".to_string()],
            ),
            (
                "lib".to_string(),
                "src/lib.rs".to_string(),
                vec!["--crate-type=lib".to_string()],
            ),
        ];

        let db = CompilationDatabase::generate_for_workspace(temp.path(), &pkgs);
        assert_eq!(db.count(), 2);

        db.write_to_file(&out_file).unwrap();
        assert!(out_file.exists());

        let loaded = CompilationDatabase::from_file(&out_file).unwrap();
        assert_eq!(loaded.count(), 2);
    }
}

use forge_core::ToolchainUtils;

#[derive(Debug, Clone)]
pub struct GoToolchain {
    pub executable: String,
    pub version: String,
}

impl GoToolchain {
    pub fn detect() -> Result<Self, String> {
        let executable = "go".to_string();
        let version = ToolchainUtils::get_tool_version(&executable, &["version"])?;

        Ok(Self {
            executable,
            version,
        })
    }

    pub fn build_args(
        &self,
        package_path: &str,
        output_binary: Option<&str>,
        tags: &[String],
        ldflags: Option<&str>,
        gcflags: Option<&str>,
    ) -> Vec<String> {
        let mut args = vec!["build".to_string()];
        if let Some(out) = output_binary {
            args.push("-o".to_string());
            args.push(out.to_string());
        }
        if !tags.is_empty() {
            args.push("-tags".to_string());
            args.push(tags.join(","));
        }
        if let Some(ld) = ldflags {
            args.push("-ldflags".to_string());
            args.push(ld.to_string());
        }
        if let Some(gc) = gcflags {
            args.push("-gcflags".to_string());
            args.push(gc.to_string());
        }
        args.push(package_path.to_string());
        args
    }

    pub fn test_args(&self, package_path: &str, tags: &[String]) -> Vec<String> {
        let mut args = vec!["test".to_string()];
        if !tags.is_empty() {
            args.push("-tags".to_string());
            args.push(tags.join(","));
        }
        args.push(package_path.to_string());
        args
    }

    pub fn vet_args(&self, package_path: &str) -> Vec<String> {
        vec!["vet".to_string(), package_path.to_string()]
    }
}

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DetectedMonorepoKind {
    CargoWorkspace,
    PnpmWorkspace,
    NpmWorkspace,
    YarnWorkspace,
    GoWork,
    GradleMultiProject,
    MavenMultiModule,
    CmakeWorkspace,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct MonorepoDiscoveryResult {
    pub kind: DetectedMonorepoKind,
    pub root: PathBuf,
    pub member_paths: Vec<PathBuf>,
    pub suggested_backend: &'static str,
}

pub struct ZeroConfigAdapter;

impl ZeroConfigAdapter {
    pub fn discover_workspace(root: &Path) -> MonorepoDiscoveryResult {
        if root.join("Cargo.toml").exists() {
            return MonorepoDiscoveryResult {
                kind: DetectedMonorepoKind::CargoWorkspace,
                root: root.to_path_buf(),
                member_paths: vec![root.join("crates")],
                suggested_backend: "rust",
            };
        }

        if root.join("pnpm-workspace.yaml").exists() {
            return MonorepoDiscoveryResult {
                kind: DetectedMonorepoKind::PnpmWorkspace,
                root: root.to_path_buf(),
                member_paths: vec![root.join("packages"), root.join("apps")],
                suggested_backend: "ts",
            };
        }

        if root.join("go.work").exists() {
            return MonorepoDiscoveryResult {
                kind: DetectedMonorepoKind::GoWork,
                root: root.to_path_buf(),
                member_paths: vec![root.to_path_buf()],
                suggested_backend: "go",
            };
        }

        if root.join("pom.xml").exists() {
            return MonorepoDiscoveryResult {
                kind: DetectedMonorepoKind::MavenMultiModule,
                root: root.to_path_buf(),
                member_paths: vec![root.to_path_buf()],
                suggested_backend: "java",
            };
        }

        MonorepoDiscoveryResult {
            kind: DetectedMonorepoKind::Unknown,
            root: root.to_path_buf(),
            member_paths: Vec::new(),
            suggested_backend: "rust",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_zero_config_adapter_discovery() {
        let temp = tempdir().unwrap();
        std::fs::write(
            temp.path().join("pnpm-workspace.yaml"),
            b"packages:
  - 'packages/*'
",
        )
        .unwrap();

        let res = ZeroConfigAdapter::discover_workspace(temp.path());
        assert_eq!(res.kind, DetectedMonorepoKind::PnpmWorkspace);
        assert_eq!(res.suggested_backend, "ts");
    }
}

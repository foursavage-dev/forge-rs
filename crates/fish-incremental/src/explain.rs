use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirtyReason {
    Clean {
        cache_key: String,
    },
    MissingOutput(PathBuf),
    SourceModified {
        path: PathBuf,
        modified_ago_secs: u64,
    },
    DependencyChanged {
        dep_name: String,
        reason: String,
    },
    EnvMismatch {
        var_name: String,
    },
    NoPreviousCache,
}

impl DirtyReason {
    pub fn description(&self, target_name: &str) -> String {
        match self {
            Self::Clean { cache_key } => {
                format!("target `{target_name}` is clean (CAS hit: {cache_key})")
            }
            Self::MissingOutput(path) => {
                format!(
                    "rebuilding `{target_name}`: output `{}` is missing",
                    path.display()
                )
            }
            Self::SourceModified {
                path,
                modified_ago_secs,
            } => {
                format!(
                    "rebuilding `{target_name}`: source `{}` was modified {modified_ago_secs}s ago",
                    path.display()
                )
            }
            Self::DependencyChanged { dep_name, reason } => {
                format!("rebuilding `{target_name}`: dependency `{dep_name}` changed ({reason})")
            }
            Self::EnvMismatch { var_name } => {
                format!("rebuilding `{target_name}`: environment variable `{var_name}` changed")
            }
            Self::NoPreviousCache => {
                format!("rebuilding `{target_name}`: no previous fingerprint record found")
            }
        }
    }
}

pub struct DirtyExplainer;

impl DirtyExplainer {
    pub fn inspect_sources(
        sources: &[PathBuf],
        last_build_time: Option<SystemTime>,
    ) -> DirtyReason {
        let now = SystemTime::now();
        for src in sources {
            if !src.exists() {
                continue;
            }
            if let Ok(meta) = fs::metadata(src)
                && let Ok(mtime) = meta.modified()
                && let Some(last_build) = last_build_time
                && mtime > last_build
            {
                let ago = now.duration_since(mtime).map(|d| d.as_secs()).unwrap_or(0);
                return DirtyReason::SourceModified {
                    path: src.clone(),
                    modified_ago_secs: ago,
                };
            }
        }
        DirtyReason::NoPreviousCache
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_explainer_descriptions() {
        let clean = DirtyReason::Clean {
            cache_key: "b3:1234".to_string(),
        };
        assert!(clean.description("core").contains("is clean"));

        let modified = DirtyReason::SourceModified {
            path: PathBuf::from("src/lib.rs"),
            modified_ago_secs: 5,
        };
        assert!(modified.description("core").contains("modified 5s ago"));
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EnvPolicy {
    Inherit,
    #[default]
    Hermetic,
    Custom(HashMap<String, String>),
}

pub fn sanitize_env(
    policy: &EnvPolicy,
    extra_vars: &HashMap<String, String>,
) -> HashMap<String, String> {
    match policy {
        EnvPolicy::Inherit => {
            let mut env: HashMap<String, String> = std::env::vars().collect();
            for (k, v) in extra_vars {
                env.insert(k.clone(), v.clone());
            }
            env
        }
        EnvPolicy::Hermetic => {
            let mut env = HashMap::new();

            env.insert("LANG".to_string(), "C".to_string());
            env.insert("LC_ALL".to_string(), "C".to_string());
            env.insert("SOURCE_DATE_EPOCH".to_string(), "0".to_string());
            env.insert("TZ".to_string(), "UTC".to_string());

            let whitelist = if cfg!(windows) {
                vec![
                    "PATH",
                    "Path",
                    "SYSTEMROOT",
                    "SystemRoot",
                    "WINDIR",
                    "windir",
                    "COMSPEC",
                    "ComSpec",
                    "TEMP",
                    "TMP",
                    "USERPROFILE",
                    "HOMEDRIVE",
                    "HOMEPATH",
                    "CARGO_HOME",
                    "RUSTUP_HOME",
                    "ProgramData",
                    "ProgramFiles",
                    "ProgramFiles(x86)",
                    "SystemDrive",
                ]
            } else {
                vec![
                    "PATH",
                    "HOME",
                    "TMPDIR",
                    "USER",
                    "CARGO_HOME",
                    "RUSTUP_HOME",
                ]
            };

            for var_name in whitelist {
                if let Ok(val) = std::env::var(var_name) {
                    env.insert(var_name.to_string(), val);
                }
            }

            for (k, v) in extra_vars {
                env.insert(k.clone(), v.clone());
            }

            env
        }
        EnvPolicy::Custom(custom) => {
            let mut env = custom.clone();
            for (k, v) in extra_vars {
                env.insert(k.clone(), v.clone());
            }
            env
        }
    }
}

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::{CcLanguage, CcOutputType};

fn clean_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else if let Some(stripped) = s.strip_prefix(r"//?/") {
        stripped.to_string()
    } else {
        s.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerFamily {
    Gcc,
    Clang,
    Msvc,
    Generic,
}

#[derive(Debug, Clone)]
pub struct CcCompiler {
    pub executable: String,
    pub family: CompilerFamily,
    pub version: String,
    pub language: CcLanguage,
}

impl CcCompiler {
    pub fn detect(language: CcLanguage) -> Result<Self, String> {
        let candidates = match language {
            CcLanguage::C => vec!["gcc", "clang", "cc", "cl"],
            CcLanguage::Cpp => vec!["g++", "clang++", "c++", "cl"],
        };

        for candidate in candidates {
            if let Ok(version) = query_compiler_version(candidate) {
                let family = if candidate.contains("clang") {
                    CompilerFamily::Clang
                } else if candidate.contains("gcc") || candidate.contains("g++") {
                    CompilerFamily::Gcc
                } else if candidate == "cl" {
                    CompilerFamily::Msvc
                } else {
                    CompilerFamily::Generic
                };

                return Ok(Self {
                    executable: candidate.to_string(),
                    family,
                    version,
                    language,
                });
            }
        }

        Err(format!(
            "No suitable C/C++ compiler found for {:?}",
            language
        ))
    }

    pub fn compile_object_args(
        &self,
        source: &Path,
        output_object: &Path,
        includes: &[PathBuf],
        flags: &[String],
        depfile: Option<&Path>,
    ) -> (String, Vec<String>) {
        let mut args = Vec::new();
        let src_clean = clean_path(source);
        let out_clean = clean_path(output_object);

        match self.family {
            CompilerFamily::Msvc => {
                args.push("/c".to_string());
                args.push(src_clean);
                args.push(format!("/Fo{out_clean}"));
                for inc in includes {
                    args.push(format!("/I{}", clean_path(inc)));
                }
                args.extend(flags.iter().cloned());
            }
            _ => {
                args.push("-c".to_string());
                args.push(src_clean);
                args.push("-o".to_string());
                args.push(out_clean);
                for inc in includes {
                    args.push("-I".to_string());
                    args.push(clean_path(inc));
                }
                if let Some(depfile) = depfile {
                    args.push("-MMD".to_string());
                    args.push("-MF".to_string());
                    args.push(clean_path(depfile));
                }
                args.extend(flags.iter().cloned());
            }
        }
        (self.executable.clone(), args)
    }

    pub fn link_args(
        &self,
        objects: &[PathBuf],
        output_path: &Path,
        ldflags: &[String],
        output_type: CcOutputType,
    ) -> (String, Vec<String>) {
        let mut args = Vec::new();
        let out_clean = clean_path(output_path);

        match output_type {
            CcOutputType::Executable => match self.family {
                CompilerFamily::Msvc => {
                    args.push(format!("/Fe{out_clean}"));
                    for obj in objects {
                        args.push(clean_path(obj));
                    }
                    args.extend(ldflags.iter().cloned());
                }
                _ => {
                    args.push("-o".to_string());
                    args.push(out_clean);
                    for obj in objects {
                        args.push(clean_path(obj));
                    }
                    args.extend(ldflags.iter().cloned());
                }
            },
            CcOutputType::StaticLib => match self.family {
                CompilerFamily::Msvc => {
                    let mut lib_args = vec![format!("/OUT:{out_clean}")];
                    for obj in objects {
                        lib_args.push(clean_path(obj));
                    }
                    return ("lib".to_string(), lib_args);
                }
                _ => {
                    let mut ar_args = vec!["rcs".to_string(), out_clean];
                    for obj in objects {
                        ar_args.push(clean_path(obj));
                    }
                    return ("ar".to_string(), ar_args);
                }
            },
            CcOutputType::SharedLib => match self.family {
                CompilerFamily::Msvc => {
                    args.push("/LD".to_string());
                    args.push(format!("/Fe{out_clean}"));
                    for obj in objects {
                        args.push(clean_path(obj));
                    }
                    args.extend(ldflags.iter().cloned());
                }
                _ => {
                    args.push("-shared".to_string());
                    args.push("-o".to_string());
                    args.push(out_clean);
                    for obj in objects {
                        args.push(clean_path(obj));
                    }
                    args.extend(ldflags.iter().cloned());
                }
            },
        }
        (self.executable.clone(), args)
    }
}

fn query_compiler_version(executable: &str) -> Result<String, String> {
    let flag = if executable == "cl" {
        "/?"
    } else {
        "--version"
    };
    let output = Command::new(executable)
        .arg(flag)
        .output()
        .map_err(|e| format!("Failed to spawn `{executable}`: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let first_line = stdout
        .lines()
        .next()
        .or_else(|| stderr.lines().next())
        .unwrap_or("unknown")
        .trim();

    Ok(first_line.to_string())
}

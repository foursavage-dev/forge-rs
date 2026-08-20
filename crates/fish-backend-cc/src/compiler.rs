use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::{CcLanguage, CcOutputType};

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
        match self.family {
            CompilerFamily::Msvc => {
                args.push("/c".to_string());
                args.push(source.to_string_lossy().to_string());
                args.push(format!("/Fo{}", output_object.to_string_lossy()));
                for inc in includes {
                    args.push(format!("/I{}", inc.to_string_lossy()));
                }
                args.extend(flags.iter().cloned());
            }
            _ => {
                args.push("-c".to_string());
                args.push(source.to_string_lossy().to_string());
                args.push("-o".to_string());
                args.push(output_object.to_string_lossy().to_string());
                for inc in includes {
                    args.push("-I".to_string());
                    args.push(inc.to_string_lossy().to_string());
                }
                if let Some(depfile) = depfile {
                    args.push("-MMD".to_string());
                    args.push("-MF".to_string());
                    args.push(depfile.to_string_lossy().to_string());
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
        match output_type {
            CcOutputType::Executable => match self.family {
                CompilerFamily::Msvc => {
                    args.push(format!("/Fe{}", output_path.to_string_lossy()));
                    for obj in objects {
                        args.push(obj.to_string_lossy().to_string());
                    }
                    args.extend(ldflags.iter().cloned());
                }
                _ => {
                    args.push("-o".to_string());
                    args.push(output_path.to_string_lossy().to_string());
                    for obj in objects {
                        args.push(obj.to_string_lossy().to_string());
                    }
                    args.extend(ldflags.iter().cloned());
                }
            },
            CcOutputType::StaticLib => match self.family {
                CompilerFamily::Msvc => {
                    let mut lib_args = vec!["/OUT:".to_string() + &output_path.to_string_lossy()];
                    for obj in objects {
                        lib_args.push(obj.to_string_lossy().to_string());
                    }
                    return ("lib".to_string(), lib_args);
                }
                _ => {
                    let mut ar_args =
                        vec!["rcs".to_string(), output_path.to_string_lossy().to_string()];
                    for obj in objects {
                        ar_args.push(obj.to_string_lossy().to_string());
                    }
                    return ("ar".to_string(), ar_args);
                }
            },
            CcOutputType::SharedLib => match self.family {
                CompilerFamily::Msvc => {
                    args.push("/LD".to_string());
                    args.push(format!("/Fe{}", output_path.to_string_lossy()));
                    for obj in objects {
                        args.push(obj.to_string_lossy().to_string());
                    }
                    args.extend(ldflags.iter().cloned());
                }
                _ => {
                    args.push("-shared".to_string());
                    args.push("-o".to_string());
                    args.push(output_path.to_string_lossy().to_string());
                    for obj in objects {
                        args.push(obj.to_string_lossy().to_string());
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

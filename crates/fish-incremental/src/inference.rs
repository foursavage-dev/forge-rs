use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct DependencyInferenceEngine;

impl DependencyInferenceEngine {
    pub fn infer_rust_imports(source: &str) -> HashSet<String> {
        let mut deps = HashSet::new();
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") || trimmed.starts_with("pub use ") {
                let rest = trimmed
                    .trim_start_matches("pub ")
                    .trim_start_matches("use ")
                    .trim();
                if let Some(crate_name) = rest.split("::").next() {
                    let clean = crate_name.trim_end_matches(';').trim();
                    if !clean.is_empty()
                        && clean != "crate"
                        && clean != "self"
                        && clean != "super"
                        && clean != "std"
                        && clean != "core"
                        && clean != "alloc"
                    {
                        deps.insert(clean.to_string());
                    }
                }
            } else if trimmed.starts_with("extern crate ") {
                let crate_name = trimmed
                    .trim_start_matches("extern crate ")
                    .trim_end_matches(';')
                    .trim();
                if !crate_name.is_empty() && crate_name != "std" {
                    deps.insert(crate_name.to_string());
                }
            }
        }
        deps
    }

    pub fn infer_ts_imports(source: &str) -> HashSet<String> {
        let mut deps = HashSet::new();
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") || trimmed.starts_with("export ") {
                if let Some(idx) = trimmed.find("from ") {
                    let module_part = trimmed[idx + 5..].trim().trim_end_matches(';').trim();
                    let clean = module_part.trim_matches('\'').trim_matches('"');
                    if !clean.starts_with('.') && !clean.starts_with('/') {
                        let pkg = if clean.starts_with('@') {
                            clean.split('/').take(2).collect::<Vec<_>>().join("/")
                        } else {
                            clean.split('/').next().unwrap_or(clean).to_string()
                        };
                        deps.insert(pkg);
                    }
                }
            } else if let Some(start) = trimmed.find("require(") {
                let rest = &trimmed[start + 8..];
                if let Some(end) = rest.find(')') {
                    let module_part = rest[..end].trim().trim_matches('\'').trim_matches('"');
                    if !module_part.starts_with('.') && !module_part.starts_with('/') {
                        deps.insert(module_part.to_string());
                    }
                }
            }
        }
        deps
    }

    pub fn infer_python_imports(source: &str) -> HashSet<String> {
        let mut deps = HashSet::new();
        for line in source.lines() {
            let trimmed = line.trim();
            if let Some(stripped) = trimmed.strip_prefix("import ") {
                for part in stripped.split(',') {
                    let first_word = part.split_whitespace().next().unwrap_or("").trim();
                    let module = first_word.split('.').next().unwrap_or("").trim();
                    if !module.is_empty() {
                        deps.insert(module.to_string());
                    }
                }
            } else if let Some(stripped) = trimmed.strip_prefix("from ")
                && let Some(idx) = stripped.find(" import ")
            {
                let first_word = stripped[..idx]
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim();
                let module = first_word.split('.').next().unwrap_or("").trim();
                if !module.is_empty() && !module.starts_with('.') {
                    deps.insert(module.to_string());
                }
            }
        }
        deps
    }

    pub fn infer_go_imports(source: &str) -> HashSet<String> {
        let mut deps = HashSet::new();
        let mut in_import_block = false;
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed == "import (" {
                in_import_block = true;
                continue;
            }
            if in_import_block {
                if trimmed == ")" {
                    in_import_block = false;
                    continue;
                }
                let clean = trimmed.trim_matches('"').trim();
                if !clean.is_empty() {
                    deps.insert(clean.to_string());
                }
            } else if let Some(stripped) = trimmed.strip_prefix("import ") {
                let clean = stripped.trim().trim_matches('"').trim();
                if !clean.is_empty() {
                    deps.insert(clean.to_string());
                }
            }
        }
        deps
    }

    pub fn scan_directory_inferred_deps(dir: &Path) -> HashSet<String> {
        let mut total_deps = HashSet::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str())
                        && let Ok(content) = fs::read_to_string(&path)
                    {
                        match ext {
                            "rs" => total_deps.extend(Self::infer_rust_imports(&content)),
                            "ts" | "tsx" | "js" | "jsx" => {
                                total_deps.extend(Self::infer_ts_imports(&content))
                            }
                            "py" => total_deps.extend(Self::infer_python_imports(&content)),
                            "go" => total_deps.extend(Self::infer_go_imports(&content)),
                            _ => {}
                        }
                    }
                } else if path.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name != "target" && name != "node_modules" && name != ".git" {
                        total_deps.extend(Self::scan_directory_inferred_deps(&path));
                    }
                }
            }
        }
        total_deps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_rust_imports() {
        let code = r#"
            use fish_core::config::Config;
            use anyhow::Result;
            use std::collections::HashMap;
            use crate::local::LocalType;
        "#;
        let deps = DependencyInferenceEngine::infer_rust_imports(code);
        assert!(deps.contains("fish_core"));
        assert!(deps.contains("anyhow"));
        assert!(!deps.contains("std"));
        assert!(!deps.contains("crate"));
    }

    #[test]
    fn test_infer_ts_imports() {
        let code = r#"
            import React, { useState } from 'react';
            import { Button } from '@shadcn/ui';
            import { helper } from './local-utils';
            const lodash = require('lodash');
        "#;
        let deps = DependencyInferenceEngine::infer_ts_imports(code);
        assert!(deps.contains("react"));
        assert!(deps.contains("@shadcn/ui"));
        assert!(deps.contains("lodash"));
        assert!(!deps.contains("./local-utils"));
    }

    #[test]
    fn test_infer_python_imports() {
        let code = r#"
            import numpy as np
            import pandas, torch
            from fastapi import FastAPI, Depends
        "#;
        let deps = DependencyInferenceEngine::infer_python_imports(code);
        assert!(deps.contains("numpy"));
        assert!(deps.contains("pandas"));
        assert!(deps.contains("torch"));
        assert!(deps.contains("fastapi"));
    }
}

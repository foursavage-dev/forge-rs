use fish_core::ToolchainUtils;

#[derive(Debug, Clone)]
pub struct ToolchainStatus {
    pub language: &'static str,
    pub detected: bool,
    pub version: Option<String>,
}

pub fn scan_toolchains() -> Vec<ToolchainStatus> {
    let check_list: &[(&str, &str, &[&str])] = &[
        ("Rust", "rustc", &["--version"]),
        ("C / C++", "clang", &["--version"]),
        ("Go", "go", &["version"]),
        ("Node / TypeScript", "node", &["--version"]),
        ("Python", "python", &["--version"]),
        ("Java", "javac", &["-version"]),
        (".NET", "dotnet", &["--version"]),
        ("Swift", "swift", &["--version"]),
        ("Dart", "dart", &["--version"]),
        ("Zig", "zig", &["version"]),
        ("Docker", "docker", &["--version"]),
    ];

    let mut results = Vec::new();

    for &(lang, binary, args) in check_list {
        let version_res = ToolchainUtils::get_tool_version(binary, args);
        let (detected, version) = match version_res {
            Ok(v) => {
                let first_line = v.lines().next().unwrap_or("").trim().to_string();
                (true, Some(first_line))
            }
            Err(_) => (false, None),
        };

        results.push(ToolchainStatus {
            language: lang,
            detected,
            version,
        });
    }

    results
}

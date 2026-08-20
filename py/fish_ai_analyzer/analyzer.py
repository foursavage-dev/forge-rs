import re
from dataclasses import dataclass, field
from typing import Dict, List, Optional

@dataclass
class FailureReport:
    error_category: str
    root_cause: str
    confidence: float
    suggested_fixes: List[str] = field(default_factory=list)
    affected_files: List[str] = field(default_factory=list)
    error_code: Optional[str] = None

class FailureAnalyzer:
    PATTERNS: Dict[str, List[str]] = {
        "COMPILATION_ERROR": [
            r"error\[E\d+\]:",
            r"syntax error",
            r"cannot find symbol",
            r"undefined reference to",
            r"SyntaxError:",
            r"TypeError:",
            r"fatal error: .* file not found",
            r"CS\d{4}:",
            r"error: unknown type name",
            r"cannot use .* as .* in assignment"
        ],
        "DEPENDENCY_ERROR": [
            r"could not resolve dependency",
            r"package .* not found",
            r"ModuleNotFoundError:",
            r"cannot find module",
            r"failed to fetch",
            r"unresolved import",
            r"NuGet package .* not found",
            r"go: .* no required module"
        ],
        "MEMORY_LIMIT": [
            r"out of memory",
            r"fatal error: runtime: out of memory",
            r"JavaScript heap out of memory",
            r"Killed.*OOM",
            r"MemoryError",
            r"std::bad_alloc"
        ],
        "TIMEOUT": [
            r"timed out after",
            r"command timeout exceeded",
            r"deadline exceeded",
            r"SIGXCPU",
            r"Execution timed out"
        ],
        "PERMISSION_ERROR": [
            r"permission denied",
            r"EACCES",
            r"Access is denied",
            r"operation not permitted"
        ],
        "TEST_FAILURE": [
            r"FAILED \(failures=\d+\)",
            r"FAIL: Test",
            r"test result: FAILED",
            r"AssertionError",
            r"Expected .* but got"
        ]
    }

    TOOLCHAIN_SUGGESTIONS: Dict[str, Dict[str, List[str]]] = {
        "rust": {
            "COMPILATION_ERROR": [
                "Run `cargo check` locally with `RUST_BACKTRACE=1`.",
                "Review borrow checker lifetime and trait bounds.",
                "Use `rustc --explain <ERROR_CODE>` for detailed diagnostics."
            ],
            "DEPENDENCY_ERROR": [
                "Check `Cargo.toml` and run `cargo update -p <crate>`.",
                "Verify workspace dependencies resolver = 2."
            ]
        },
        "go": {
            "COMPILATION_ERROR": [
                "Run `go vet ./...` to detect structural errors.",
                "Verify type signatures and imported package identifiers."
            ],
            "DEPENDENCY_ERROR": [
                "Run `go mod tidy` or `go mod download`.",
                "Verify module path in `go.mod` matches remote repository."
            ]
        },
        "typescript": {
            "COMPILATION_ERROR": [
                "Run `tsc --noEmit` to verify type definitions.",
                "Check `tsconfig.json` compilerOptions strict settings."
            ],
            "DEPENDENCY_ERROR": [
                "Run `npm install` or `pnpm install` to sync lockfile.",
                "Verify `@types/*` declarations are installed in devDependencies."
            ]
        },
        "python": {
            "COMPILATION_ERROR": [
                "Run `mypy` or `pyright` for static type checking.",
                "Check Python syntax compatibility for Python 3.10+."
            ],
            "DEPENDENCY_ERROR": [
                "Verify virtual environment is activated.",
                "Run `pip install -r requirements.txt`."
            ]
        },
        "cc": {
            "COMPILATION_ERROR": [
                "Verify include paths and header file availability.",
                "Check compile_commands.json configuration."
            ]
        },
        "docker": {
            "COMPILATION_ERROR": [
                "Verify Dockerfile base image availability.",
                "Check multi-stage build target names and file copy paths."
            ]
        }
    }

    def analyze(self, toolchain: str, stderr: str, stdout: str = "", exit_code: int = 1) -> FailureReport:
        combined_logs = f"{stderr}\n{stdout}"
        
        detected_category = "UNKNOWN_ERROR"
        matched_pattern = ""
        confidence = 0.5
        error_code = None

        code_match = re.search(r'(?:error\[(E\d+)\]|CS(\d{4})|TS(\d{4}))', combined_logs)
        if code_match:
            error_code = next(g for g in code_match.groups() if g is not None)

        for category, patterns in self.PATTERNS.items():
            for p in patterns:
                if re.search(p, combined_logs, re.IGNORECASE):
                    detected_category = category
                    matched_pattern = p
                    confidence = 0.95 if error_code else 0.88
                    break
            if detected_category != "UNKNOWN_ERROR":
                break

        suggested_fixes = self._generate_suggestions(detected_category, toolchain)
        affected_files = self._extract_files(combined_logs)

        return FailureReport(
            error_category=detected_category,
            root_cause=f"Detected pattern '{matched_pattern}' in {toolchain} logs",
            confidence=confidence,
            suggested_fixes=suggested_fixes,
            affected_files=affected_files,
            error_code=error_code
        )

    def _generate_suggestions(self, category: str, toolchain: str) -> List[str]:
        tc_dict = self.TOOLCHAIN_SUGGESTIONS.get(toolchain.lower(), {})
        if category in tc_dict:
            return tc_dict[category]
            
        defaults = {
            "COMPILATION_ERROR": ["Check source syntax for compiler errors.", "Run local compiler diagnostic."],
            "DEPENDENCY_ERROR": ["Verify package manifest and lockfile integrity.", "Ensure package registry connectivity."],
            "MEMORY_LIMIT": ["Increase memory limit in fish.toml.", "Reduce build parallelism with --jobs."],
            "TIMEOUT": ["Increase timeout_ms in task configuration.", "Inspect slow test suites or deadlocks."],
            "PERMISSION_ERROR": ["Check file permissions in build output directories."],
            "TEST_FAILURE": ["Run failing test locally in isolation with verbose logging."]
        }
        return defaults.get(category, ["Inspect full execution logs with --verbose."])

    def _extract_files(self, logs: str) -> List[str]:
        file_pattern = r'([a-zA-Z0-9_\-./\\]+\.(?:rs|go|ts|js|py|cpp|c|h|hpp|java|cs|swift|dart|zig|dockerfile|toml|json))'
        found = set(re.findall(file_pattern, logs, re.IGNORECASE))
        return sorted(list(found))[:8]

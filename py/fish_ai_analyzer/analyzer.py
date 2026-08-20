import re
from dataclasses import dataclass, field
from typing import List

@dataclass
class FailureReport:
    error_category: str
    root_cause: str
    confidence: float
    suggested_fixes: List[str] = field(default_factory=list)
    affected_files: List[str] = field(default_factory=list)

class FailureAnalyzer:
    PATTERNS = {
        "COMPILATION_ERROR": [
            r"error\[E\d+\]:",
            r"syntax error",
            r"cannot find symbol",
            r"undefined reference to",
            r"SyntaxError:",
            r"TypeError:"
        ],
        "DEPENDENCY_ERROR": [
            r"could not resolve dependency",
            r"package .* not found",
            r"ModuleNotFoundError:",
            r"cannot find module",
            r"failed to fetch"
        ],
        "MEMORY_LIMIT": [
            r"out of memory",
            r"fatal error: runtime: out of memory",
            r"JavaScript heap out of memory",
            r"Killed.*OOM"
        ],
        "TIMEOUT": [
            r"timed out after",
            r"command timeout exceeded",
            r"deadline exceeded"
        ],
        "PERMISSION_ERROR": [
            r"permission denied",
            r"EACCES",
            r"Access is denied"
        ]
    }

    def analyze(self, toolchain: str, stderr: str, stdout: str = "", exit_code: int = 1) -> FailureReport:
        combined_logs = f"{stderr}\n{stdout}"
        
        detected_category = "UNKNOWN_ERROR"
        matched_pattern = ""
        confidence = 0.5

        for category, patterns in self.PATTERNS.items():
            for p in patterns:
                if re.search(p, combined_logs, re.IGNORECASE):
                    detected_category = category
                    matched_pattern = p
                    confidence = 0.92
                    break
            if detected_category != "UNKNOWN_ERROR":
                break

        suggested_fixes = self._generate_suggestions(detected_category, toolchain, combined_logs)
        affected_files = self._extract_files(combined_logs)

        return FailureReport(
            error_category=detected_category,
            root_cause=f"Detected pattern '{matched_pattern}' in {toolchain} build logs",
            confidence=confidence,
            suggested_fixes=suggested_fixes,
            affected_files=affected_files
        )

    def _generate_suggestions(self, category: str, toolchain: str, logs: str) -> List[str]:
        if category == "COMPILATION_ERROR":
            return [
                f"Check source syntax for {toolchain} compile errors.",
                "Run linter / compiler diagnostics locally with --explain."
            ]
        elif category == "DEPENDENCY_ERROR":
            return [
                "Verify lockfile integrity and network connectivity.",
                "Check package version constraints in manifest."
            ]
        elif category == "MEMORY_LIMIT":
            return [
                "Increase task memory allocation in fish.toml.",
                "Reduce parallel job concurrency with --jobs."
            ]
        elif category == "TIMEOUT":
            return [
                "Increase timeout_ms in task configuration.",
                "Inspect heavy test suites or deadlocks."
            ]
        elif category == "PERMISSION_ERROR":
            return [
                "Check output directory write permissions.",
                "Ensure sandbox execution user has write access."
            ]
        return ["Inspect full debug logs with --verbose."]

    def _extract_files(self, logs: str) -> List[str]:
        file_pattern = r'([a-zA-Z0-9_\-./\\]+\.(?:rs|go|ts|js|py|cpp|c|java|cs|swift|dart|zig))'
        found = set(re.findall(file_pattern, logs))
        return sorted(list(found))[:5]

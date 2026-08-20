from typing import Dict, Any, List

class AiBenchmarkSuite:
    def __init__(self):
        self.test_cases = [
            {
                "prompt": "Create a polyglot workspace manifest with Rust backend and TypeScript frontend.",
                "expected_keys": ["workspace", "package", "backend", "pipelines"]
            },
            {
                "prompt": "Configure remote cache with S3 and TLS token authentication.",
                "expected_keys": ["remote", "cache_url", "token"]
            },
            {
                "prompt": "Add custom Starlark build rule for code generation.",
                "expected_keys": ["rule", "inputs", "outputs", "cmd"]
            }
        ]

    def evaluate_manifest_generation(self, generated_manifest: str, test_case_index: int = 0) -> Dict[str, Any]:
        if test_case_index >= len(self.test_cases):
            return {"score": 0.0, "passed": False, "error": "Index out of range"}

        case = self.test_cases[test_case_index]
        matched_keys = [k for k in case["expected_keys"] if k in generated_manifest]
        accuracy = len(matched_keys) / len(case["expected_keys"])

        return {
            "test_case": case["prompt"],
            "accuracy": accuracy,
            "passed": accuracy >= 0.75,
            "matched_keys": matched_keys,
            "missing_keys": [k for k in case["expected_keys"] if k not in matched_keys]
        }

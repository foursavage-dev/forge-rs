import dataclasses
import json
import sys
from typing import Dict, Any

from fish_ai_analyzer.analyzer import FailureAnalyzer
from fish_analytics.metrics import BuildAnalytics, BuildRunMetrics
from fish_optimizer.scheduler_opt import ScheduleOptimizer

from .autofix import AiAutoFixer
from .benchmarks import AiBenchmarkSuite
from .prewarmer import PredictiveBuildPrewarmer
from .risk_scorer import PredictivePrRiskScorer
from .semantic_impact import SemanticImpactAnalyzer
from .test_reorder import SmartTestReorderer

class FishAIServer:
    def __init__(self):
        self.analyzer = FailureAnalyzer()
        self.optimizer = ScheduleOptimizer()
        self.analytics = BuildAnalytics()
        self.autofixer = AiAutoFixer()
        self.benchmarks = AiBenchmarkSuite()
        self.prewarmer = PredictiveBuildPrewarmer()
        self.risk_scorer = PredictivePrRiskScorer()
        self.semantic_impact = SemanticImpactAnalyzer()
        self.test_reorderer = SmartTestReorderer()

    def handle_request(self, req: Dict[str, Any]) -> Dict[str, Any]:
        method = req.get("method")
        params = req.get("params", {})
        req_id = req.get("id")

        if method == "ping":
            return {"jsonrpc": "2.0", "id": req_id, "result": {"status": "pong"}}

        elif method in ("analyze_failure", "analyze_error"):
            toolchain = params.get("toolchain", "rust")
            stderr = params.get("stderr", "")
            stdout = params.get("stdout", "")
            exit_code = params.get("exit_code", 1)
            report = self.analyzer.analyze(toolchain, stderr, stdout, exit_code)
            return {"jsonrpc": "2.0", "id": req_id, "result": dataclasses.asdict(report)}

        elif method == "optimize_schedule":
            dependencies = params.get("dependencies", {})
            durations = params.get("durations", {})
            workers = params.get("max_workers", 4)
            plan = self.optimizer.optimize_schedule(dependencies, durations, max_workers=workers)
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "critical_path": plan.critical_path,
                    "estimated_speedup": plan.estimated_speedup,
                    "ordered_tasks": plan.ordered_tasks,
                }
            }

        elif method == "record_run":
            metrics = BuildRunMetrics(
                run_id=params.get("run_id", "run-0"),
                total_duration_ms=params.get("total_duration_ms", 0),
                tasks_count=params.get("tasks_count", 0),
                cache_hits=params.get("cache_hits", 0),
                cache_misses=params.get("cache_misses", 0),
                failed_count=params.get("failed_count", 0),
                bottleneck_tasks=params.get("bottlenecks", [])
            )
            self.analytics.record_run(metrics)
            return {"jsonrpc": "2.0", "id": req_id, "result": {"status": "recorded"}}

        elif method == "analytics_summary":
            return {"jsonrpc": "2.0", "id": req_id, "result": self.analytics.summary()}

        elif method == "autofix":
            content = params.get("file_content", "")
            msg = params.get("error_message", "")
            res = self.autofixer.generate_fix(content, msg)
            return {"jsonrpc": "2.0", "id": req_id, "result": res}

        elif method == "benchmark_ai":
            manifest = params.get("manifest", "")
            case_idx = params.get("index", 0)
            res = self.benchmarks.evaluate_manifest_generation(manifest, case_idx)
            return {"jsonrpc": "2.0", "id": req_id, "result": res}

        elif method == "predict_pr_risk":
            files = params.get("files", [])
            added = params.get("lines_added", 0)
            deleted = params.get("lines_deleted", 0)
            res = self.risk_scorer.compute_pr_risk(files, added, deleted)
            return {"jsonrpc": "2.0", "id": req_id, "result": res}

        elif method == "semantic_impact":
            files = params.get("files", [])
            symbols = params.get("symbols", [])
            res = self.semantic_impact.analyze_semantic_impact(files, symbols)
            return {"jsonrpc": "2.0", "id": req_id, "result": res}

        elif method == "test_reorder":
            tests = params.get("tests", [])
            res = self.test_reorderer.prioritize_tests(tests)
            return {"jsonrpc": "2.0", "id": req_id, "result": res}

        elif method == "prewarm_cache":
            active_file = params.get("active_file", "")
            symbol = params.get("symbol", "")
            res = self.prewarmer.on_keystroke_activity(active_file, symbol)
            return {"jsonrpc": "2.0", "id": req_id, "result": res}

        else:
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "error": {"code": -32601, "message": f"Method {method} not found"}
            }

def main():
    server = FishAIServer()
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            req = json.loads(line)
            resp = server.handle_request(req)
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()
        except Exception as e:
            err_resp = {
                "jsonrpc": "2.0",
                "id": None,
                "error": {"code": -32700, "message": str(e)}
            }
            sys.stdout.write(json.dumps(err_resp) + "\n")
            sys.stdout.flush()

if __name__ == "__main__":
    main()

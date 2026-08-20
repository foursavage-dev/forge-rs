import json
import sys
from typing import Any, Dict

from fish_ai_analyzer.analyzer import FailureAnalyzer
from fish_optimizer.scheduler_opt import ScheduleOptimizer
from fish_analytics.metrics import BuildAnalytics, BuildRunMetrics
from fish_recommender.recommender import SmartRecommender

class FishAIServer:
    def __init__(self):
        self.analyzer = FailureAnalyzer()
        self.optimizer = ScheduleOptimizer()
        self.analytics = BuildAnalytics()
        self.recommender = SmartRecommender()

    def handle_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        method = request.get("method")
        params = request.get("params", {})
        req_id = request.get("id")

        try:
            if method == "analyze_failure":
                res = self.analyzer.analyze(
                    toolchain=params.get("toolchain", "rust"),
                    stderr=params.get("stderr", ""),
                    stdout=params.get("stdout", ""),
                    exit_code=params.get("exit_code", 1)
                )
                result = {
                    "error_category": res.error_category,
                    "root_cause": res.root_cause,
                    "confidence": res.confidence,
                    "suggested_fixes": res.suggested_fixes,
                    "affected_files": res.affected_files,
                    "error_code": res.error_code
                }
            elif method == "optimize_schedule":
                plan = self.optimizer.optimize_schedule(
                    task_dependencies=params.get("dependencies", {}),
                    historical_durations=params.get("durations", {}),
                    task_memory_mb=params.get("memory_mb", {}),
                    max_workers=params.get("max_workers", 8),
                    memory_budget_mb=params.get("memory_budget_mb", 8192)
                )
                result = {
                    "critical_path": plan.critical_path,
                    "ordered_tasks": plan.ordered_tasks,
                    "estimated_speedup": plan.estimated_speedup,
                    "max_concurrent_memory_mb": plan.max_concurrent_memory_mb,
                    "parallel_batches": plan.parallel_batches
                }
            elif method == "recommend_tasks":
                affected = self.recommender.recommend_tasks(
                    changed_files=params.get("changed_files", []),
                    file_to_package_map=params.get("file_to_package_map", {}),
                    package_dependencies=params.get("package_dependencies", {})
                )
                flaky = self.recommender.detect_flaky_candidates(
                    task_history=params.get("task_history", {})
                )
                result = {
                    "recommended_tasks": affected,
                    "flaky_candidates": flaky
                }
            elif method == "record_run":
                run = BuildRunMetrics(
                    run_id=params.get("run_id", "run_1"),
                    total_duration_ms=params.get("total_duration_ms", 0),
                    tasks_count=params.get("tasks_count", 0),
                    cache_hits=params.get("cache_hits", 0),
                    cache_misses=params.get("cache_misses", 0),
                    failed_count=params.get("failed_count", 0),
                    bottleneck_tasks=params.get("bottleneck_tasks", []),
                    task_durations_ms=params.get("task_durations_ms", {}),
                    memory_peak_mb=params.get("memory_peak_mb", 0)
                )
                self.analytics.record_run(run)
                result = {"status": "recorded", "run_id": run.run_id}
            elif method == "analytics_summary":
                result = self.analytics.summary()
            elif method == "ping":
                result = {"status": "pong", "engine": "fish-ai"}
            else:
                return {
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "error": {"code": -32601, "message": f"Method '{method}' not found"}
                }

            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": result
            }
        except Exception as e:
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "error": {"code": -32000, "message": str(e)}
            }

    def run_stdio(self):
        for line in sys.stdin:
            line = line.strip()
            if not line:
                continue
            try:
                req = json.loads(line)
                resp = self.handle_request(req)
                sys.stdout.write(json.dumps(resp) + "\n")
                sys.stdout.flush()
            except Exception as e:
                err_resp = {
                    "jsonrpc": "2.0",
                    "id": None,
                    "error": {"code": -32700, "message": f"Parse error: {e}"}
                }
                sys.stdout.write(json.dumps(err_resp) + "\n")
                sys.stdout.flush()

if __name__ == "__main__":
    server = FishAIServer()
    server.run_stdio()

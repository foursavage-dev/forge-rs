import math
from dataclasses import dataclass, field
from typing import Any, Dict, List

@dataclass
class BuildRunMetrics:
    run_id: str
    total_duration_ms: int
    tasks_count: int
    cache_hits: int
    cache_misses: int
    failed_count: int
    bottleneck_tasks: List[str] = field(default_factory=list)
    task_durations_ms: Dict[str, int] = field(default_factory=dict)
    memory_peak_mb: int = 0

class BuildAnalytics:
    def __init__(self):
        self.runs: List[BuildRunMetrics] = []

    def record_run(self, run: BuildRunMetrics):
        self.runs.append(run)

    def calculate_cache_efficiency(self) -> float:
        if not self.runs:
            return 0.0
        total_hits = sum(r.cache_hits for r in self.runs)
        total_requests = sum(r.cache_hits + r.cache_misses for r in self.runs)
        if total_requests == 0:
            return 0.0
        return round((total_hits / total_requests) * 100.0, 2)

    def calculate_failure_rate(self) -> float:
        if not self.runs:
            return 0.0
        total_failed = sum(r.failed_count for r in self.runs)
        total_tasks = sum(r.tasks_count for r in self.runs)
        if total_tasks == 0:
            return 0.0
        return round((total_failed / total_tasks) * 100.0, 2)

    def estimate_time_saved_ms(self, avg_task_cost_ms: int = 1500) -> int:
        total_hits = sum(r.cache_hits for r in self.runs)
        return total_hits * avg_task_cost_ms

    def calculate_percentiles(self, values: List[int]) -> Dict[str, float]:
        if not values:
            return {"p50": 0.0, "p90": 0.0, "p95": 0.0, "p99": 0.0, "avg": 0.0, "min": 0.0, "max": 0.0}
        
        sorted_vals = sorted(values)
        n = len(sorted_vals)

        def get_pct(p: float) -> float:
            idx = int(math.ceil(p * n)) - 1
            return float(sorted_vals[max(0, min(idx, n - 1))])

        return {
            "min": float(sorted_vals[0]),
            "max": float(sorted_vals[-1]),
            "avg": round(sum(sorted_vals) / n, 2),
            "p50": get_pct(0.50),
            "p90": get_pct(0.90),
            "p95": get_pct(0.95),
            "p99": get_pct(0.99)
        }

    def duration_percentiles(self) -> Dict[str, float]:
        durations = [r.total_duration_ms for r in self.runs]
        return self.calculate_percentiles(durations)

    def identify_global_bottlenecks(self, top_n: int = 3) -> List[str]:
        frequency: Dict[str, int] = {}
        for r in self.runs:
            for b in r.bottleneck_tasks:
                frequency[b] = frequency.get(b, 0) + 1
        sorted_bottlenecks = sorted(frequency.items(), key=lambda x: x[1], reverse=True)
        return [task for task, _ in sorted_bottlenecks[:top_n]]

    def summary(self) -> Dict[str, Any]:
        return {
            "total_runs": len(self.runs),
            "cache_efficiency_pct": self.calculate_cache_efficiency(),
            "failure_rate_pct": self.calculate_failure_rate(),
            "time_saved_ms": self.estimate_time_saved_ms(),
            "durations": self.duration_percentiles(),
            "top_bottlenecks": self.identify_global_bottlenecks()
        }

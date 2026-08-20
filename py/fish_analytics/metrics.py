from dataclasses import dataclass, field
from typing import Dict, List

@dataclass
class BuildRunMetrics:
    run_id: str
    total_duration_ms: int
    tasks_count: int
    cache_hits: int
    cache_misses: int
    failed_count: int
    bottleneck_tasks: List[str] = field(default_factory=list)

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

    def identify_global_bottlenecks(self, top_n: int = 3) -> List[str]:
        frequency: Dict[str, int] = {}
        for r in self.runs:
            for b in r.bottleneck_tasks:
                frequency[b] = frequency.get(b, 0) + 1
        sorted_bottlenecks = sorted(frequency.items(), key=lambda x: x[1], reverse=True)
        return [task for task, _ in sorted_bottlenecks[:top_n]]

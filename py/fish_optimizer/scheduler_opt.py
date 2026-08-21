from dataclasses import dataclass, field
from typing import Dict, List, Set, Tuple

@dataclass
class OptimizationPlan:
    critical_path: List[str]
    ordered_tasks: List[str]
    estimated_speedup: float
    max_concurrent_memory_mb: int
    parallel_batches: List[List[str]] = field(default_factory=list)

class ScheduleOptimizer:
    def optimize_schedule(
        self,
        task_dependencies: Dict[str, List[str]],
        historical_durations: Dict[str, float],
        task_memory_mb: Dict[str, int] = None,
        max_workers: int = 8,
        memory_budget_mb: int = 8192
    ) -> OptimizationPlan:
        task_memory_mb = task_memory_mb or {}
        task_weights: Dict[str, float] = {}
        
        all_tasks = set(task_dependencies.keys())
        for deps in task_dependencies.values():
            all_tasks.update(deps)

        for task in all_tasks:
            task_weights[task] = historical_durations.get(task, 1.0)

        critical_path = self._calculate_critical_path(task_dependencies, task_weights)
        
        in_degree: Dict[str, int] = {t: 0 for t in all_tasks}
        dependents: Dict[str, List[str]] = {t: [] for t in all_tasks}

        for parent, deps in task_dependencies.items():
            for child in deps:
                dependents[child].append(parent)
                in_degree[parent] += 1

        ready = [t for t, deg in in_degree.items() if deg == 0]
        ready.sort(key=lambda t: task_weights[t], reverse=True)
        
        ordered: List[str] = []
        batches: List[List[str]] = []
        current_batch: List[str] = []
        current_mem = 0

        while ready:
            next_ready: List[str] = []
            current_batch = []
            current_mem = 0
            
            for t in ready:
                mem = task_memory_mb.get(t, 256)
                if len(current_batch) < max_workers and (current_mem + mem <= memory_budget_mb or not current_batch):
                    current_batch.append(t)
                    current_mem += mem
                    ordered.append(t)
                else:
                    next_ready.append(t)

            for executed in current_batch:
                for nxt in dependents[executed]:
                    in_degree[nxt] -= 1
                    if in_degree[nxt] == 0:
                        next_ready.append(nxt)

            batches.append(current_batch)
            next_ready.sort(key=lambda t: task_weights[t], reverse=True)
            ready = next_ready

        sequential_time = sum(task_weights.values())
        critical_time = sum(task_weights.get(t, 1.0) for t in critical_path) or 1.0
        theoretical_parallel = max(critical_time, sequential_time / max(1, max_workers))
        speedup = sequential_time / max(0.1, theoretical_parallel)

        max_mem_used = max((sum(task_memory_mb.get(t, 256) for t in b) for b in batches), default=256)

        return OptimizationPlan(
            critical_path=critical_path,
            ordered_tasks=ordered,
            estimated_speedup=round(speedup, 2),
            max_concurrent_memory_mb=max_mem_used,
            parallel_batches=batches
        )

    def _calculate_critical_path(
        self,
        dependencies: Dict[str, List[str]],
        weights: Dict[str, float]
    ) -> List[str]:
        memo: Dict[str, float] = {}
        next_hop: Dict[str, str] = {}

        def get_longest(task: str) -> float:
            if task in memo:
                return memo[task]
            deps = dependencies.get(task, [])
            if not deps:
                memo[task] = weights.get(task, 1.0)
                return memo[task]
            
            max_child_cost = 0.0
            best_child = ""
            for d in deps:
                c = get_longest(d)
                if c > max_child_cost:
                    max_child_cost = c
                    best_child = d
            
            memo[task] = weights.get(task, 1.0) + max_child_cost
            if best_child:
                next_hop[task] = best_child
            return memo[task]

        all_roots = set(dependencies.keys())
        all_children = {c for deps in dependencies.values() for c in deps}
        top_roots = all_roots - all_children or all_roots

        best_root = max(top_roots, key=get_longest) if top_roots else ""
        path: List[str] = []
        curr = best_root
        while curr:
            path.append(curr)
            curr = next_hop.get(curr, "")

        return path

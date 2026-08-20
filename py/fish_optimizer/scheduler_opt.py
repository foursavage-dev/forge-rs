from dataclasses import dataclass, field
from typing import Dict, List, Set

@dataclass
class OptimizationPlan:
    critical_path: List[str]
    ordered_tasks: List[str]
    estimated_speedup: float

class ScheduleOptimizer:
    def optimize_schedule(
        self,
        task_dependencies: Dict[str, List[str]],
        historical_durations: Dict[str, float],
        max_workers: int = 8
    ) -> OptimizationPlan:
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
        while ready:
            curr = ready.pop(0)
            ordered.append(curr)
            for nxt in dependents[curr]:
                in_degree[nxt] -= 1
                if in_degree[nxt] == 0:
                    ready.append(nxt)
            ready.sort(key=lambda t: task_weights[t], reverse=True)

        sequential_time = sum(task_weights.values())
        critical_time = sum(task_weights.get(t, 1.0) for t in critical_path) or 1.0
        theoretical_parallel = max(critical_time, sequential_time / max(1, max_workers))
        speedup = sequential_time / max(0.1, theoretical_parallel)

        return OptimizationPlan(
            critical_path=critical_path,
            ordered_tasks=ordered,
            estimated_speedup=round(speedup, 2)
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

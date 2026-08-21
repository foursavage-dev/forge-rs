from collections import deque
from typing import Dict, List, Set

class SmartRecommender:
    def recommend_tasks(
        self,
        changed_files: List[str],
        file_to_package_map: Dict[str, str],
        package_dependencies: Dict[str, List[str]]
    ) -> List[str]:
        direct_affected: Set[str] = set()
        
        for file_path in changed_files:
            for pattern, pkg in file_to_package_map.items():
                if pattern in file_path:
                    direct_affected.add(pkg)

        reverse_deps: Dict[str, List[str]] = {}
        for parent, deps in package_dependencies.items():
            for d in deps:
                reverse_deps.setdefault(d, []).append(parent)

        all_affected: Set[str] = set(direct_affected)
        queue = deque(direct_affected)

        while queue:
            current = queue.popleft()
            for dependent in reverse_deps.get(current, []):
                if dependent not in all_affected:
                    all_affected.add(dependent)
                    queue.append(dependent)

        return sorted(list(all_affected))

    def detect_flaky_candidates(
        self,
        task_history: Dict[str, List[bool]],
        min_flips: int = 2
    ) -> List[str]:
        flaky: List[str] = []
        for task, history in task_history.items():
            if len(history) < 3:
                continue
            flips = 0
            for i in range(1, len(history)):
                if history[i] != history[i - 1]:
                    flips += 1
            if flips >= min_flips:
                flaky.append(task)
        return sorted(flaky)

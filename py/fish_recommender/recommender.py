from typing import Dict, List, Set

class SmartRecommender:
    def recommend_tasks(
        self,
        changed_files: List[str],
        file_to_package_map: Dict[str, str],
        package_dependencies: Dict[str, List[str]]
    ) -> List[str]:
        affected_packages: Set[str] = set()
        
        for file_path in changed_files:
            for pattern, pkg in file_to_package_map.items():
                if pattern in file_path:
                    affected_packages.add(pkg)

        all_affected = set(affected_packages)
        for parent, deps in package_dependencies.items():
            for aff in affected_packages:
                if aff in deps:
                    all_affected.add(parent)

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

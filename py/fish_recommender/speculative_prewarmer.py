from typing import List, Dict, Set

class SpeculativePrewarmer:
    def __init__(self, dep_graph: Dict[str, List[str]]):
        self.dep_graph = dep_graph

    def find_dependent_targets(self, changed_files: List[str]) -> List[str]:
        # Reverse adjacency, so dependents of a node can be found without
        # scanning the whole graph on every step.
        reverse: Dict[str, List[str]] = {}
        for node, deps in self.dep_graph.items():
            for d in deps:
                reverse.setdefault(d, []).append(node)

        # Nodes directly consuming a changed file/directory.
        impacted: Set[str] = set()
        for f in changed_files:
            for node, deps in self.dep_graph.items():
                if f in deps or any(f.startswith(d) for d in deps):
                    impacted.add(node)

        # Propagate through the full transitive closure of dependents. The
        # previous implementation only walked one level, so a change deep in
        # the graph missed its grandparents.
        queue: List[str] = list(impacted)
        while queue:
            current = queue.pop()
            for dependent in reverse.get(current, []):
                if dependent not in impacted:
                    impacted.add(dependent)
                    queue.append(dependent)

        return sorted(impacted)

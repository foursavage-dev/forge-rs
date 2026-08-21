from typing import List, Dict, Set

class SpeculativePrewarmer:
    def __init__(self, dep_graph: Dict[str, List[str]]):
        self.dep_graph = dep_graph

    def find_dependent_targets(self, changed_files: List[str]) -> List[str]:
        impacted: Set[str] = set()
        for f in changed_files:
            for node, deps in self.dep_graph.items():
                if f in deps or any(f.startswith(d) for d in deps):
                    impacted.add(node)
                    
        for node in list(impacted):
            for parent, deps in self.dep_graph.items():
                if node in deps:
                    impacted.add(parent)
                    
        return sorted(list(impacted))

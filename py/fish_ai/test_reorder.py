from typing import List, Dict, Any

class SmartTestReorderer:
    def __init__(self):
        pass

    def prioritize_tests(self, test_list: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        return sorted(
            test_list,
            key=lambda t: (t.get("recent_failures", 0) * 100) + (1000.0 / max(1, t.get("duration_ms", 1))),
            reverse=True
        )

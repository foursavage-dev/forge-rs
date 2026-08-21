from typing import Dict, List, Any

class FlakyQuarantine:
    def __init__(self, threshold_score: float = 0.25):
        self.threshold = threshold_score
        self.records: Dict[str, List[bool]] = {}

    def record_test_run(self, test_id: str, passed: bool):
        if test_id not in self.records:
            self.records[test_id] = []
        self.records[test_id].append(passed)

    def calculate_flakiness_score(self, test_id: str) -> float:
        runs = self.records.get(test_id, [])
        if len(runs) < 2:
            return 0.0
        
        switches = 0
        for i in range(1, len(runs)):
            if runs[i] != runs[i - 1]:
                switches += 1
        
        return round(switches / (len(runs) - 1), 3)

    def should_quarantine(self, test_id: str) -> bool:
        return self.calculate_flakiness_score(test_id) >= self.threshold

    def get_quarantined_tests(self) -> List[str]:
        return [t for t in self.records if self.should_quarantine(t)]

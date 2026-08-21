import math
from typing import Dict, List, Any

class BuildTimePredictor:
    def __init__(self, alpha: float = 0.3):
        self.alpha = alpha
        self.history: Dict[str, List[float]] = {}
        self.coefficients: Dict[str, float] = {}

    def record_run(self, task_name: str, duration_sec: float, source_bytes: int = 1000):
        if task_name not in self.history:
            self.history[task_name] = []
        self.history[task_name].append(duration_sec)
        
        rate = duration_sec / max(1, source_bytes)
        if task_name in self.coefficients:
            self.coefficients[task_name] = (self.alpha * rate) + ((1.0 - self.alpha) * self.coefficients[task_name])
        else:
            self.coefficients[task_name] = rate

    def predict_duration(self, task_name: str, source_bytes: int = 1000) -> float:
        if task_name in self.coefficients:
            return round(self.coefficients[task_name] * source_bytes, 3)
        if task_name in self.history and self.history[task_name]:
            return round(sum(self.history[task_name]) / len(self.history[task_name]), 3)
        return round(0.001 * source_bytes, 3)

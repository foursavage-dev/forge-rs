from typing import Dict, List, Any

class AutonomousOptimizer:
    def __init__(self):
        self.best_configurations: Dict[str, Dict[str, Any]] = {}

    def evaluate_build_profile(
        self,
        target_name: str,
        flags: List[str],
        duration_sec: float,
        binary_size_bytes: int
    ) -> float:
        # Guard against a zero/negative duration so an instant or unmeasured
        # build cannot raise ZeroDivisionError; the score still favours fast
        # and small profiles.
        effective_duration = max(duration_sec, 1e-6)
        size_mb = max(1, binary_size_bytes / 1024 / 1024)
        efficiency_score = 1000.0 / (effective_duration * size_mb)

        current_best = self.best_configurations.get(target_name)
        if current_best is None or efficiency_score > current_best["score"]:
            self.best_configurations[target_name] = {
                "flags": flags,
                "duration_sec": duration_sec,
                "binary_size": binary_size_bytes,
                "score": efficiency_score
            }
            
        return efficiency_score

    def suggest_optimal_flags(self, target_name: str) -> List[str]:
        if target_name in self.best_configurations:
            return self.best_configurations[target_name]["flags"]
        return ["-O3", "-flto", "--codegen-units=1"]

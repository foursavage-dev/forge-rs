from typing import List, Dict, Any

class PredictiveBuildPrewarmer:
    def __init__(self):
        self.typing_history: List[str] = []

    def on_keystroke_activity(self, active_file: str, dirty_symbol: str) -> Dict[str, Any]:
        self.typing_history.append(active_file)
        if len(self.typing_history) > 20:
            self.typing_history.pop(0)

        predicted_targets = []
        if active_file.endswith(".rs"):
            predicted_targets.append("cargo-check")
            predicted_targets.append("fish-backend-rust")
        elif active_file.endswith(".ts") or active_file.endswith(".js"):
            predicted_targets.append("tsc-check")
            predicted_targets.append("fish-backend-ts")
        elif active_file.endswith(".go"):
            predicted_targets.append("go-vet")

        return {
            "status": "PREWARM_SCHEDULED",
            "active_file": active_file,
            "dirty_symbol": dirty_symbol,
            "prewarm_targets": predicted_targets
        }

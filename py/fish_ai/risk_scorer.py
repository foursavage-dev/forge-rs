from typing import List, Dict, Any

class PredictivePrRiskScorer:
    def __init__(self):
        self.critical_files = [
            "Cargo.lock",
            "package-lock.json",
            "pnpm-lock.yaml",
            "fish.toml",
            "fish.yaml",
            "CMakeLists.txt"
        ]

    def compute_pr_risk(self, changed_files: List[str], lines_added: int, lines_deleted: int) -> Dict[str, Any]:
        risk_score = 0.0
        risk_factors = []

        for f in changed_files:
            for crit in self.critical_files:
                if f.endswith(crit):
                    risk_score += 0.35
                    risk_factors.append(f"Critical dependency file modified: {f}")

        total_churn = lines_added + lines_deleted
        if total_churn > 1000:
            risk_score += 0.30
            risk_factors.append(f"High code churn detected ({total_churn} lines)")
        elif total_churn > 300:
            risk_score += 0.15

        if len(changed_files) > 15:
            risk_score += 0.25
            risk_factors.append(f"Large breadth of affected packages ({len(changed_files)} files)")

        risk_score = min(1.0, max(0.0, risk_score))

        if risk_score > 0.65:
            severity = "HIGH"
            suggested_action = "Run full workspace test matrix and quarantine sensitive tests."
        elif risk_score > 0.35:
            severity = "MEDIUM"
            suggested_action = "Execute targeted affected package tests."
        else:
            severity = "LOW"
            suggested_action = "Standard fast CI validation."

        return {
            "risk_score": round(risk_score, 2),
            "severity": severity,
            "risk_factors": risk_factors,
            "suggested_action": suggested_action
        }

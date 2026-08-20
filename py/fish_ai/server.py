import json
import sys
from typing import Dict, Any

from .analyzer import FishAiAnalyzer
from .benchmarks import AiBenchmarkSuite
from .prewarmer import PredictiveBuildPrewarmer
from .risk_scorer import PredictivePrRiskScorer

analyzer = FishAiAnalyzer()
benchmark_suite = AiBenchmarkSuite()
prewarmer = PredictiveBuildPrewarmer()
risk_scorer = PredictivePrRiskScorer()

def handle_rpc_request(req: Dict[str, Any]) -> Dict[str, Any]:
    method = req.get("method")
    params = req.get("params", {})
    req_id = req.get("id")

    if method == "analyze_error":
        toolchain = params.get("toolchain", "rust")
        stderr = params.get("stderr", "")
        exit_code = params.get("exit_code", 1)
        res = analyzer.analyze(toolchain, stderr, exit_code)
        return {"jsonrpc": "2.0", "id": req_id, "result": res}

    elif method == "benchmark_ai":
        manifest = params.get("manifest", "")
        case_idx = params.get("index", 0)
        res = benchmark_suite.evaluate_manifest_generation(manifest, case_idx)
        return {"jsonrpc": "2.0", "id": req_id, "result": res}

    elif method == "predict_pr_risk":
        files = params.get("files", [])
        added = params.get("lines_added", 0)
        deleted = params.get("lines_deleted", 0)
        res = risk_scorer.compute_pr_risk(files, added, deleted)
        return {"jsonrpc": "2.0", "id": req_id, "result": res}

    elif method == "prewarm_cache":
        active_file = params.get("active_file", "")
        symbol = params.get("symbol", "")
        res = prewarmer.on_keystroke_activity(active_file, symbol)
        return {"jsonrpc": "2.0", "id": req_id, "result": res}

    else:
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "error": {"code": -32601, "message": f"Method {method} not found"}
        }

def main():
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            req = json.loads(line)
            resp = handle_rpc_request(req)
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()
        except Exception as e:
            err_resp = {
                "jsonrpc": "2.0",
                "id": None,
                "error": {"code": -32700, "message": str(e)}
            }
            sys.stdout.write(json.dumps(err_resp) + "\n")
            sys.stdout.flush()

if __name__ == "__main__":
    main()

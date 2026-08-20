import unittest
from fish_ai.server import FishAIServer

class TestFishAIServer(unittest.TestCase):
    def setUp(self):
        self.server = FishAIServer()

    def test_ping(self):
        req = {"jsonrpc": "2.0", "id": 1, "method": "ping"}
        resp = self.server.handle_request(req)
        self.assertEqual(resp["result"]["status"], "pong")

    def test_analyze_failure_rpc(self):
        req = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "analyze_failure",
            "params": {
                "toolchain": "rust",
                "stderr": "error[E0308]: mismatched types\n --> src/lib.rs:20:9",
                "exit_code": 1
            }
        }
        resp = self.server.handle_request(req)
        self.assertEqual(resp["result"]["error_category"], "COMPILATION_ERROR")
        self.assertEqual(resp["result"]["error_code"], "E0308")

    def test_optimize_schedule_rpc(self):
        req = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "optimize_schedule",
            "params": {
                "dependencies": {"a": ["b"], "b": []},
                "durations": {"a": 5.0, "b": 10.0}
            }
        }
        resp = self.server.handle_request(req)
        self.assertIn("ordered_tasks", resp["result"])
        self.assertEqual(resp["result"]["ordered_tasks"], ["b", "a"])

    def test_record_run_and_summary_rpc(self):
        req_record = {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "record_run",
            "params": {
                "run_id": "build-42",
                "total_duration_ms": 2500,
                "tasks_count": 8,
                "cache_hits": 6,
                "cache_misses": 2,
                "failed_count": 0
            }
        }
        resp = self.server.handle_request(req_record)
        self.assertEqual(resp["result"]["status"], "recorded")

        req_sum = {
            "jsonrpc": "2.0",
            "id": 5,
            "method": "analytics_summary",
            "params": {}
        }
        resp_sum = self.server.handle_request(req_sum)
        self.assertEqual(resp_sum["result"]["total_runs"], 1)
        self.assertEqual(resp_sum["result"]["cache_efficiency_pct"], 75.0)

if __name__ == "__main__":
    unittest.main()

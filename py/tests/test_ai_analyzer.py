import unittest
from fish_ai_analyzer.analyzer import FailureAnalyzer

class TestFailureAnalyzer(unittest.TestCase):
    def setUp(self):
        self.analyzer = FailureAnalyzer()

    def test_rust_compilation_error(self):
        stderr = "error[E0425]: cannot find value `x` in this scope\n --> src/main.rs:10:5"
        report = self.analyzer.analyze("rust", stderr, "", 1)
        self.assertEqual(report.error_category, "COMPILATION_ERROR")
        self.assertIn("src/main.rs", report.affected_files)
        self.assertGreaterEqual(report.confidence, 0.9)

    def test_memory_limit_detection(self):
        stderr = "fatal error: runtime: out of memory"
        report = self.analyzer.analyze("go", stderr, "", 137)
        self.assertEqual(report.error_category, "MEMORY_LIMIT")
        self.assertTrue(len(report.suggested_fixes) > 0)

if __name__ == '__main__':
    unittest.main()

import unittest
from fish_analytics.metrics import BuildAnalytics, BuildRunMetrics

class TestBuildAnalytics(unittest.TestCase):
    def test_cache_efficiency_and_summary(self):
        analytics = BuildAnalytics()
        analytics.record_run(BuildRunMetrics("run-1", 1000, 10, 8, 2, 0, ["heavy_task"]))
        analytics.record_run(BuildRunMetrics("run-2", 1200, 10, 6, 4, 1, ["heavy_task", "link_task"]))
        
        eff = analytics.calculate_cache_efficiency()
        self.assertEqual(eff, 70.0)
        bottlenecks = analytics.identify_global_bottlenecks(1)
        self.assertEqual(bottlenecks, ["heavy_task"])

        saved = analytics.estimate_time_saved_ms(1000)
        self.assertEqual(saved, 14000)

        fail_rate = analytics.calculate_failure_rate()
        self.assertEqual(fail_rate, 5.0)

        pcts = analytics.duration_percentiles()
        self.assertEqual(pcts["min"], 1000.0)
        self.assertEqual(pcts["max"], 1200.0)

        summary = analytics.summary()
        self.assertEqual(summary["total_runs"], 2)
        self.assertEqual(summary["cache_efficiency_pct"], 70.0)

if __name__ == '__main__':
    unittest.main()

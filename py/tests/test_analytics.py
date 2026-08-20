import unittest
from fish_analytics.metrics import BuildAnalytics, BuildRunMetrics

class TestBuildAnalytics(unittest.TestCase):
    def test_cache_efficiency(self):
        analytics = BuildAnalytics()
        analytics.record_run(BuildRunMetrics("run-1", 1000, 10, 8, 2, 0, ["heavy_task"]))
        analytics.record_run(BuildRunMetrics("run-2", 1200, 10, 6, 4, 0, ["heavy_task", "link_task"]))
        
        eff = analytics.calculate_cache_efficiency()
        self.assertEqual(eff, 70.0)
        bottlenecks = analytics.identify_global_bottlenecks(1)
        self.assertEqual(bottlenecks, ["heavy_task"])

if __name__ == '__main__':
    unittest.main()

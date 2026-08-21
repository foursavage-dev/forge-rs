import unittest
from fish_optimizer.scheduler_opt import ScheduleOptimizer

class TestScheduleOptimizer(unittest.TestCase):
    def setUp(self):
        self.optimizer = ScheduleOptimizer()

    def test_optimization_plan(self):
        dependencies = {
            "app": ["core", "utils"],
            "core": ["db"],
            "utils": [],
            "db": []
        }
        durations = {
            "db": 10.0,
            "core": 5.0,
            "utils": 2.0,
            "app": 3.0
        }
        plan = self.optimizer.optimize_schedule(dependencies, durations, max_workers=4)
        self.assertEqual(plan.critical_path, ["app", "core", "db"])
        self.assertGreater(plan.estimated_speedup, 1.0)
        self.assertEqual(len(plan.ordered_tasks), 4)

if __name__ == '__main__':
    unittest.main()

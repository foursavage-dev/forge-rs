import unittest
from fish_optimizer.autonomous_optimizer import AutonomousOptimizer
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

    def test_cyclic_dependencies_raise_instead_of_recursing_forever(self):
        dependencies = {"a": ["b"], "b": ["a"]}
        durations = {"a": 1.0, "b": 1.0}
        with self.assertRaises(ValueError):
            self.optimizer.optimize_schedule(dependencies, durations)

    def test_self_dependency_raises(self):
        dependencies = {"x": ["x"]}
        with self.assertRaises(ValueError):
            self.optimizer.optimize_schedule(dependencies, {"x": 1.0})


class TestAutonomousOptimizer(unittest.TestCase):
    def test_zero_duration_build_does_not_crash(self):
        optimizer = AutonomousOptimizer()
        score = optimizer.evaluate_build_profile(
            "target", ["-O2"], duration_sec=0.0, binary_size_bytes=1024 * 1024
        )
        self.assertGreater(score, 0.0)

    def test_best_flags_are_remembered(self):
        optimizer = AutonomousOptimizer()
        optimizer.evaluate_build_profile("t", ["-O0"], 10.0, 1024)
        optimizer.evaluate_build_profile("t", ["-O3"], 5.0, 1024)
        self.assertEqual(optimizer.suggest_optimal_flags("t"), ["-O3"])

if __name__ == '__main__':
    unittest.main()

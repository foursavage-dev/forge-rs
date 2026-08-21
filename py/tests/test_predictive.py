import unittest
from fish_optimizer.build_time_predictor import BuildTimePredictor
from fish_recommender.flaky_quarantine import FlakyQuarantine
from fish_recommender.speculative_prewarmer import SpeculativePrewarmer

class TestPredictiveAlgorithms(unittest.TestCase):
    def test_build_time_predictor(self):
        predictor = BuildTimePredictor()
        predictor.record_run("core_build", 10.0, 1000)
        predictor.record_run("core_build", 12.0, 1000)
        
        predicted = predictor.predict_duration("core_build", 2000)
        self.assertGreater(predicted, 15.0)

    def test_flaky_quarantine(self):
        quarantine = FlakyQuarantine(threshold_score=0.3)
        quarantine.record_test_run("test_network", True)
        quarantine.record_test_run("test_network", False)
        quarantine.record_test_run("test_network", True)
        quarantine.record_test_run("test_network", False)
        
        score = quarantine.calculate_flakiness_score("test_network")
        self.assertEqual(score, 1.0)
        self.assertTrue(quarantine.should_quarantine("test_network"))
        self.assertIn("test_network", quarantine.get_quarantined_tests())

    def test_speculative_prewarmer(self):
        graph = {
            "app": ["crates/fish-core", "crates/fish-graph"],
            "crates/fish-graph": ["crates/fish-core"],
            "crates/fish-core": []
        }
        prewarmer = SpeculativePrewarmer(graph)
        impacted = prewarmer.find_dependent_targets(["crates/fish-core/src/lib.rs"])
        self.assertIn("app", impacted)
        self.assertIn("crates/fish-graph", impacted)

if __name__ == "__main__":
    unittest.main()

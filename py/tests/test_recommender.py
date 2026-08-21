import unittest
from fish_recommender.recommender import SmartRecommender

class TestSmartRecommender(unittest.TestCase):
    def setUp(self):
        self.recommender = SmartRecommender()

    def test_affected_packages(self):
        changed = ["packages/core/src/index.ts"]
        mapping = {"packages/core": "core", "packages/web": "web"}
        deps = {"web": ["core"]}
        affected = self.recommender.recommend_tasks(changed, mapping, deps)
        self.assertIn("core", affected)
        self.assertIn("web", affected)

    def test_flaky_detection(self):
        history = {
            "test_stable": [True, True, True, True],
            "test_flaky": [True, False, True, False]
        }
        flaky = self.recommender.detect_flaky_candidates(history)
        self.assertEqual(flaky, ["test_flaky"])

if __name__ == '__main__':
    unittest.main()

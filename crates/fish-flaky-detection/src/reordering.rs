#[derive(Debug, Clone)]
pub struct TestCaseMeta {
    pub name: String,
    pub recent_failure_count: u32,
    pub avg_duration_ms: u64,
    pub complexity_score: f64,
}

#[derive(Debug, Clone, Default)]
pub struct SmartTestReorderer;

impl SmartTestReorderer {
    pub fn compute_priority_score(meta: &TestCaseMeta) -> f64 {
        let failure_weight = (meta.recent_failure_count as f64) * 100.0;
        let speed_bonus = 1000.0 / (meta.avg_duration_ms.max(1) as f64);
        failure_weight + speed_bonus + meta.complexity_score * 10.0
    }

    pub fn reorder_tests(test_cases: &[TestCaseMeta]) -> Vec<TestCaseMeta> {
        let mut sorted = test_cases.to_vec();
        sorted.sort_by(|a, b| {
            let score_a = Self::compute_priority_score(a);
            let score_b = Self::compute_priority_score(b);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_test_reordering_prioritizes_failing_fast_tests() {
        let t1 = TestCaseMeta {
            name: "test_slow_stable".to_string(),
            recent_failure_count: 0,
            avg_duration_ms: 5000,
            complexity_score: 1.0,
        };
        let t2 = TestCaseMeta {
            name: "test_fast_failing".to_string(),
            recent_failure_count: 3,
            avg_duration_ms: 50,
            complexity_score: 2.0,
        };

        let reordered = SmartTestReorderer::reorder_tests(&[t1, t2]);
        assert_eq!(reordered[0].name, "test_fast_failing");
    }
}

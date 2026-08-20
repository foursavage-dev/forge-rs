use forge_graph::NodeId;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TaskTimingEstimate {
    pub node_id: NodeId,
    pub label: String,
    pub estimated_duration: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct AgentBucket {
    pub agent_id: usize,
    pub total_duration: Duration,
    pub task_ids: Vec<NodeId>,
}

pub struct DteBinPacker;

impl DteBinPacker {
    pub fn partition_tasks(tasks: &[TaskTimingEstimate], agent_count: usize) -> Vec<AgentBucket> {
        let count = agent_count.max(1);
        let mut buckets: Vec<AgentBucket> = (0..count)
            .map(|id| AgentBucket {
                agent_id: id,
                total_duration: Duration::ZERO,
                task_ids: Vec::new(),
            })
            .collect();

        let mut sorted_tasks = tasks.to_vec();
        sorted_tasks.sort_by_key(|b| std::cmp::Reverse(b.estimated_duration));

        for task in sorted_tasks {
            buckets.sort_by(|a, b| {
                a.total_duration
                    .cmp(&b.total_duration)
                    .then_with(|| a.agent_id.cmp(&b.agent_id))
            });
            buckets[0].task_ids.push(task.node_id);
            buckets[0].total_duration += task.estimated_duration;
        }

        buckets.sort_by_key(|b| b.agent_id);
        buckets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dte_bin_packing_balance() {
        let mut graph = forge_graph::BuildGraph::new();
        let n1 = graph.add_node("t1");
        let n2 = graph.add_node("t2");
        let n3 = graph.add_node("t3");
        let n4 = graph.add_node("t4");

        let tasks = vec![
            TaskTimingEstimate {
                node_id: n1,
                label: "t1".to_string(),
                estimated_duration: Duration::from_secs(10),
            },
            TaskTimingEstimate {
                node_id: n2,
                label: "t2".to_string(),
                estimated_duration: Duration::from_secs(8),
            },
            TaskTimingEstimate {
                node_id: n3,
                label: "t3".to_string(),
                estimated_duration: Duration::from_secs(6),
            },
            TaskTimingEstimate {
                node_id: n4,
                label: "t4".to_string(),
                estimated_duration: Duration::from_secs(4),
            },
        ];

        let buckets = DteBinPacker::partition_tasks(&tasks, 2);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].total_duration, Duration::from_secs(14));
        assert_eq!(buckets[1].total_duration, Duration::from_secs(14));
    }
}

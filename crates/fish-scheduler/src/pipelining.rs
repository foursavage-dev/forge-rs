use fish_graph::NodeId;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineStage {
    Pending,
    MetadataReady,
    ObjectReady,
    ArtifactLinked,
}

pub struct PipelinedCompilationCoordinator {
    node_stages: HashMap<NodeId, PipelineStage>,
}

impl PipelinedCompilationCoordinator {
    pub fn new() -> Self {
        Self {
            node_stages: HashMap::new(),
        }
    }

    pub fn set_stage(&mut self, node_id: NodeId, stage: PipelineStage) {
        self.node_stages.insert(node_id, stage);
    }

    pub fn get_stage(&self, node_id: NodeId) -> PipelineStage {
        self.node_stages
            .get(&node_id)
            .copied()
            .unwrap_or(PipelineStage::Pending)
    }

    pub fn can_start_compilation(&self, dependencies: &[NodeId]) -> bool {
        dependencies.iter().all(|&dep| {
            let stage = self.get_stage(dep);
            matches!(
                stage,
                PipelineStage::MetadataReady
                    | PipelineStage::ObjectReady
                    | PipelineStage::ArtifactLinked
            )
        })
    }

    pub fn can_start_linking(&self, direct_node: NodeId, dependencies: &[NodeId]) -> bool {
        let direct_stage = self.get_stage(direct_node);
        if direct_stage != PipelineStage::ObjectReady
            && direct_stage != PipelineStage::ArtifactLinked
        {
            return false;
        }

        dependencies.iter().all(|&dep| {
            let stage = self.get_stage(dep);
            stage == PipelineStage::ArtifactLinked
        })
    }

    pub fn ready_pipelined_nodes(
        &self,
        nodes: &[NodeId],
        deps_map: &HashMap<NodeId, Vec<NodeId>>,
    ) -> Vec<NodeId> {
        let mut ready = Vec::new();
        let active_set: HashSet<NodeId> = self
            .node_stages
            .iter()
            .filter(|(_, s)| **s != PipelineStage::Pending)
            .map(|(id, _)| *id)
            .collect();

        for &node in nodes {
            if active_set.contains(&node) {
                continue;
            }
            if let Some(deps) = deps_map.get(&node) {
                if self.can_start_compilation(deps) {
                    ready.push(node);
                }
            } else {
                ready.push(node);
            }
        }
        ready
    }
}

impl Default for PipelinedCompilationCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipelined_compilation_unblocks_on_metadata() {
        let mut coordinator = PipelinedCompilationCoordinator::new();
        let dep = NodeId::from(0);
        let downstream = NodeId::from(1);

        assert!(!coordinator.can_start_compilation(&[dep]));

        coordinator.set_stage(dep, PipelineStage::MetadataReady);
        assert!(coordinator.can_start_compilation(&[dep]));

        coordinator.set_stage(downstream, PipelineStage::ObjectReady);
        assert!(!coordinator.can_start_linking(downstream, &[dep]));

        coordinator.set_stage(dep, PipelineStage::ArtifactLinked);
        assert!(coordinator.can_start_linking(downstream, &[dep]));
    }
}

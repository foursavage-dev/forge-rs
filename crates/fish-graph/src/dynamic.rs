use crate::error::GraphError;
use crate::graph::{BuildGraph, NodeId};

#[derive(Debug, Clone)]
pub struct DynamicTaskSpec<T> {
    pub payload: T,
    pub dependencies: Vec<NodeId>,
}

pub struct DynamicGraphExpander<'a, T> {
    graph: &'a mut BuildGraph<T>,
}

impl<'a, T> DynamicGraphExpander<'a, T> {
    pub fn new(graph: &'a mut BuildGraph<T>) -> Self {
        Self { graph }
    }

    pub fn expand_node(
        &mut self,
        parent: NodeId,
        new_tasks: Vec<DynamicTaskSpec<T>>,
    ) -> Result<Vec<NodeId>, GraphError> {
        let mut created_ids = Vec::with_capacity(new_tasks.len());

        for task in new_tasks {
            let child_id = self.graph.add_node(task.payload);
            self.graph.add_dependency(parent, child_id)?;
            for dep in task.dependencies {
                self.graph.add_dependency(dep, child_id)?;
            }
            created_ids.push(child_id);
        }

        Ok(created_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_graph_expansion() {
        let mut graph = BuildGraph::new();
        let generator = graph.add_node("code_gen");

        let mut expander = DynamicGraphExpander::new(&mut graph);
        let dynamic_tasks = vec![
            DynamicTaskSpec {
                payload: "gen_part_1",
                dependencies: vec![],
            },
            DynamicTaskSpec {
                payload: "gen_part_2",
                dependencies: vec![],
            },
        ];

        let children = expander.expand_node(generator, dynamic_tasks).unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(graph.len(), 3);
        assert_eq!(graph.deps(children[0]).unwrap(), &[generator]);
        assert_eq!(graph.deps(children[1]).unwrap(), &[generator]);
    }
}

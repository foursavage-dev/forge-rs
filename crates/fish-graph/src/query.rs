use crate::graph::{BuildGraph, NodeId};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryExpr {
    Target(String),
    All,
    Deps(Box<QueryExpr>),
    Rdeps(Box<QueryExpr>),
    AllPaths(Box<QueryExpr>, Box<QueryExpr>),
    SomePath(Box<QueryExpr>, Box<QueryExpr>),
    Filter(String, Box<QueryExpr>),
    Union(Box<QueryExpr>, Box<QueryExpr>),
    Intersect(Box<QueryExpr>, Box<QueryExpr>),
}

pub struct GraphQueryEngine<'a, T> {
    graph: &'a BuildGraph<T>,
    name_resolver: Box<dyn Fn(&T) -> String + 'a>,
}

impl<'a, T> GraphQueryEngine<'a, T> {
    pub fn new<F>(graph: &'a BuildGraph<T>, name_resolver: F) -> Self
    where
        F: Fn(&T) -> String + 'a,
    {
        Self {
            graph,
            name_resolver: Box::new(name_resolver),
        }
    }

    pub fn eval(&self, expr: &QueryExpr) -> HashSet<NodeId> {
        match expr {
            QueryExpr::Target(name) => self
                .graph
                .nodes()
                .iter()
                .filter(|n| (self.name_resolver)(&n.payload) == *name)
                .map(|n| n.id)
                .collect(),
            QueryExpr::All => self.graph.nodes().iter().map(|n| n.id).collect(),
            QueryExpr::Deps(inner) => {
                let targets = self.eval(inner);
                let mut result = HashSet::new();
                for target in targets {
                    self.collect_transitive_deps(target, &mut result);
                }
                result
            }
            QueryExpr::Rdeps(inner) => {
                let targets = self.eval(inner);
                let mut result = HashSet::new();
                for target in targets {
                    self.collect_transitive_rdeps(target, &mut result);
                }
                result
            }
            QueryExpr::AllPaths(from_expr, to_expr) => {
                let from_set = self.eval(from_expr);
                let to_set = self.eval(to_expr);
                let mut path_nodes = HashSet::new();
                for &from in &from_set {
                    for &to in &to_set {
                        self.find_all_path_nodes(from, to, &mut path_nodes);
                    }
                }
                path_nodes
            }
            QueryExpr::SomePath(from_expr, to_expr) => {
                let from_set = self.eval(from_expr);
                let to_set = self.eval(to_expr);
                for &from in &from_set {
                    for &to in &to_set {
                        if let Some(path) = self.find_shortest_path(from, to) {
                            return path.into_iter().collect();
                        }
                    }
                }
                HashSet::new()
            }
            QueryExpr::Filter(pattern, inner) => {
                let set = self.eval(inner);
                set.into_iter()
                    .filter(|id| {
                        if let Some(node) = self.graph.node(*id) {
                            let name = (self.name_resolver)(&node.payload);
                            name.contains(pattern)
                        } else {
                            false
                        }
                    })
                    .collect()
            }
            QueryExpr::Union(left, right) => {
                let l = self.eval(left);
                let r = self.eval(right);
                l.union(&r).copied().collect()
            }
            QueryExpr::Intersect(left, right) => {
                let l = self.eval(left);
                let r = self.eval(right);
                l.intersection(&r).copied().collect()
            }
        }
    }

    fn collect_transitive_deps(&self, start: NodeId, visited: &mut HashSet<NodeId>) {
        if !visited.insert(start) {
            return;
        }
        if let Ok(deps) = self.graph.deps(start) {
            for &dep in deps {
                self.collect_transitive_deps(dep, visited);
            }
        }
    }

    fn collect_transitive_rdeps(&self, start: NodeId, visited: &mut HashSet<NodeId>) {
        if !visited.insert(start) {
            return;
        }
        if let Ok(rdeps) = self.graph.dependents(start) {
            for &rdep in rdeps {
                self.collect_transitive_rdeps(rdep, visited);
            }
        }
    }

    fn find_all_path_nodes(
        &self,
        current: NodeId,
        target: NodeId,
        result: &mut HashSet<NodeId>,
    ) -> bool {
        if current == target {
            result.insert(current);
            return true;
        }
        let mut on_path = false;
        if let Ok(deps) = self.graph.deps(current) {
            for &dep in deps {
                if self.find_all_path_nodes(dep, target, result) {
                    result.insert(current);
                    on_path = true;
                }
            }
        }
        on_path
    }

    fn find_shortest_path(&self, from: NodeId, to: NodeId) -> Option<Vec<NodeId>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = std::collections::HashMap::new();

        queue.push_back(from);
        visited.insert(from);

        while let Some(current) = queue.pop_front() {
            if current == to {
                let mut path = vec![to];
                let mut curr = to;
                while let Some(&p) = parent.get(&curr) {
                    path.push(p);
                    curr = p;
                }
                path.reverse();
                return Some(path);
            }

            if let Ok(deps) = self.graph.deps(current) {
                for &dep in deps {
                    if visited.insert(dep) {
                        parent.insert(dep, current);
                        queue.push_back(dep);
                    }
                }
            }
        }

        None
    }
}

pub fn parse_query(input: &str) -> Result<QueryExpr, String> {
    let trimmed = input.trim();
    if trimmed == "//..." || trimmed == "*" {
        return Ok(QueryExpr::All);
    }
    if let Some(inner) = trimmed
        .strip_prefix("deps(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let parsed_inner = parse_query(inner)?;
        return Ok(QueryExpr::Deps(Box::new(parsed_inner)));
    }
    if let Some(inner) = trimmed
        .strip_prefix("rdeps(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let parsed_inner = parse_query(inner)?;
        return Ok(QueryExpr::Rdeps(Box::new(parsed_inner)));
    }
    if let Some(args) = trimmed
        .strip_prefix("allpaths(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let left = parse_query(parts[0])?;
            let right = parse_query(parts[1])?;
            return Ok(QueryExpr::AllPaths(Box::new(left), Box::new(right)));
        }
    }
    if let Some(args) = trimmed
        .strip_prefix("somepath(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let left = parse_query(parts[0])?;
            let right = parse_query(parts[1])?;
            return Ok(QueryExpr::SomePath(Box::new(left), Box::new(right)));
        }
    }
    if let Some(args) = trimmed
        .strip_prefix("filter(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let parts: Vec<&str> = args.splitn(2, ',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let pattern = parts[0].trim_matches('"').trim_matches('\'').to_string();
            let inner = parse_query(parts[1])?;
            return Ok(QueryExpr::Filter(pattern, Box::new(inner)));
        }
    }

    let target_name = trimmed.trim_start_matches("//").to_string();
    Ok(QueryExpr::Target(target_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_query_engine_deps_and_rdeps() {
        let mut graph = BuildGraph::new();
        let u = graph.add_node("util");
        let c = graph.add_node("core");
        let a = graph.add_node("app");

        graph.add_dependency(u, c).unwrap();
        graph.add_dependency(c, a).unwrap();

        let engine = GraphQueryEngine::new(&graph, |&s| s.to_string());

        let deps_query = parse_query("deps(app)").unwrap();
        let deps = engine.eval(&deps_query);
        assert_eq!(deps.len(), 3);
        assert!(deps.contains(&a));
        assert!(deps.contains(&c));
        assert!(deps.contains(&u));

        let rdeps_query = parse_query("rdeps(util)").unwrap();
        let rdeps = engine.eval(&rdeps_query);
        assert_eq!(rdeps.len(), 3);
        assert!(rdeps.contains(&a));
    }

    #[test]
    fn test_graph_query_allpaths_and_filter() {
        let mut graph = BuildGraph::new();
        let u = graph.add_node("util");
        let b = graph.add_node("backend");
        let a = graph.add_node("app");

        graph.add_dependency(u, b).unwrap();
        graph.add_dependency(b, a).unwrap();

        let engine = GraphQueryEngine::new(&graph, |&s| s.to_string());

        let paths_query = parse_query("allpaths(app, util)").unwrap();
        let path_nodes = engine.eval(&paths_query);
        assert_eq!(path_nodes.len(), 3);

        let filter_query = parse_query("filter('back', deps(app))").unwrap();
        let filtered = engine.eval(&filter_query);
        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains(&b));
    }
}

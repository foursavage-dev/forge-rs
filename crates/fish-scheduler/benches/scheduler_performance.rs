use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use fish_graph::BuildGraph;
use std::hint::black_box;

fn build_synthetic_dag(node_count: usize) -> BuildGraph<String> {
    let mut graph = BuildGraph::new();
    let mut node_ids = Vec::with_capacity(node_count);

    for i in 0..node_count {
        let id = graph.add_node(format!("task_{i}"));
        node_ids.push(id);
    }

    for i in 1..node_count {
        let parent = node_ids[i / 2];
        let child = node_ids[i];
        let _ = graph.add_dependency(parent, child);
    }

    graph
}

fn bench_graph_topological_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("dag_topological_sort");

    for size in [50, 200, 1000] {
        let graph = build_synthetic_dag(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &graph, |b, g| {
            b.iter(|| {
                let order = g.topological_order();
                black_box(order);
            });
        });
    }

    group.finish();
}

fn bench_graph_ready_nodes(c: &mut Criterion) {
    let mut group = c.benchmark_group("dag_ready_nodes");

    for size in [50, 200, 1000] {
        let graph = build_synthetic_dag(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &graph, |b, g| {
            b.iter(|| {
                let ready = g.ready_nodes();
                black_box(ready);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_graph_topological_sort,
    bench_graph_ready_nodes
);
criterion_main!(benches);

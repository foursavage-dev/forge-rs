#![forbid(unsafe_code)]

use crate::metrics::{BuildMetrics, TaskMetrics};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlamegraphNode {
    pub name: String,
    pub value: u64,
    pub children: Vec<FlamegraphNode>,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlamegraphData {
    pub root: FlamegraphNode,
    pub total_duration_ms: u64,
    pub task_count: usize,
}

pub struct FlamegraphGenerator;

impl FlamegraphGenerator {
    pub fn from_build_metrics(metrics: &BuildMetrics) -> FlamegraphData {
        let mut builder = FlamegraphBuilder::new();
        
        for task in &metrics.tasks {
            builder.add_task(task);
        }
        
        builder.build()
    }
    
    pub fn generate_svg(fg: &FlamegraphData) -> String {
        // Simple SVG generation for flamegraph
        let width = 800;
        let height = fg.task_count as u32 * 30 + 50;
        let mut svg = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
<style>
    .task {{ fill: #4CAF50; stroke: #333; stroke-width: 1; }}
    .task-cached {{ fill: #2196F3; stroke: #333; stroke-width: 1; }}
    .task-failed {{ fill: #f44336; stroke: #333; stroke-width: 1; }}
    .text {{ font-family: Arial, sans-serif; font-size: 12px; fill: white; }}
</style>
<rect width="100%" height="100%" fill="#f5f5f5"/>
"#,
            width, height
        );
        
        let mut y = 10;
        let total_duration = fg.total_duration_ms as f64;
        
        fn render_node(node: &FlamegraphNode, x: f64, y: &mut u32, total_duration: f64, svg: &mut String, depth: usize) {
            let width = if total_duration > 0.0 {
                (node.value as f64 / total_duration * 780.0).max(20.0)
            } else {
                20.0
            };
            
            let color_class = if node.name.contains("cached") {
                "task-cached"
            } else if node.name.contains("failed") {
                "task-failed"
            } else {
                "task"
            };
            
            *svg += &format!(
                r#"<rect x="{}" y="{}" width="{}" height="25" class="{}" />
<text x="{}" y="{}" class="text">{}</text>
"#,
                x + 10, y, width, color_class, x + 15, y + 17, node.name
            );
            
            *y += 30;
            
            let mut child_x = x;
            for child in &node.children {
                render_node(child, child_x, y, total_duration, svg, depth + 1);
                child_x += if total_duration > 0.0 {
                    child.value as f64 / total_duration * 780.0
                } else {
                    20.0
                };
            }
        }
        
        render_node(&fg.root, 0.0, &mut y, total_duration, &mut svg, 0);
        
        svg += "</svg>";
        svg
    }
}

struct FlamegraphBuilder {
    nodes: HashMap<String, FlamegraphNode>,
    root: FlamegraphNode,
    total_duration: u64,
    task_count: usize,
}

impl FlamegraphBuilder {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: FlamegraphNode {
                name: "build".to_string(),
                value: 0,
                children: Vec::new(),
                depth: 0,
            },
            total_duration: 0,
            task_count: 0,
        }
    }
    
    fn add_task(&mut self, task: &TaskMetrics) {
        let duration = task.duration_ms.unwrap_or(0);
        self.total_duration += duration;
        self.task_count += 1;
        
        let node = FlamegraphNode {
            name: task.description.clone(),
            value: duration,
            children: Vec::new(),
            depth: 0,
        };
        
        self.nodes.insert(task.task_id.clone(), node);
    }
    
    fn build(mut self) -> FlamegraphData {
        // Build tree structure from dependencies
        let mut orphan_nodes: Vec<String> = self.nodes.keys().cloned().collect();
        
        for (task_id, node) in &self.nodes {
            // In a real implementation, we would build the dependency tree
            // For now, we'll just add all tasks as children of root
            self.root.children.push(node.clone());
        }
        
        self.root.value = self.total_duration;
        
        FlamegraphData {
            root: self.root,
            total_duration_ms: self.total_duration,
            task_count: self.task_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::BuildStatus;
    
    #[test]
    fn test_flamegraph_generation() {
        let mut metrics = BuildMetrics::new("test-1".to_string(), "test".to_string(), "rust".to_string());
        
        let mut task = TaskMetrics::new("task-1".to_string(), "compile".to_string());
        task.complete(BuildStatus::Success, false);
        metrics.add_task(task);
        
        let fg = FlamegraphGenerator::from_build_metrics(&metrics);
        assert_eq!(fg.task_count, 1);
    }
    
    #[test]
    fn test_svg_generation() {
        let fg = FlamegraphData {
            root: FlamegraphNode {
                name: "test".to_string(),
                value: 1000,
                children: Vec::new(),
                depth: 0,
            },
            total_duration_ms: 1000,
            task_count: 1,
        };
        
        let svg = FlamegraphGenerator::generate_svg(&fg);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("test"));
    }
}

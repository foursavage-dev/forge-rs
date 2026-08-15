#![forbid(unsafe_code)]

//! Real-time build monitoring and dashboard
//! 
//! This module provides live monitoring capabilities for build operations,
//! including progress tracking, status updates, and metrics visualization.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct TaskProgress {
    pub name: String,
    pub status: BuildStatus,
    pub progress: f64,
    pub message: String,
    pub start_time: Option<Instant>,
    pub duration: Option<Duration>,
}

pub struct BuildMonitor {
    tasks: Arc<spin::Mutex<HashMap<String, TaskProgress>>>,
    total_tasks: AtomicU64,
    completed_tasks: AtomicU64,
    failed_tasks: AtomicU64,
    is_running: Arc<AtomicBool>,
    start_time: Instant,
}

impl BuildMonitor {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(spin::Mutex::new(HashMap::new())),
            total_tasks: AtomicU64::new(0),
            completed_tasks: AtomicU64::new(0),
            failed_tasks: AtomicU64::new(0),
            is_running: Arc::new(AtomicBool::new(true)),
            start_time: Instant::now(),
        }
    }

    pub fn add_task(&self, name: String) {
        self.total_tasks.fetch_add(1, Ordering::SeqCst);
        let mut tasks = self.tasks.lock();
        tasks.insert(name.clone(), TaskProgress {
            name,
            status: BuildStatus::Pending,
            progress: 0.0,
            message: "Waiting to start".to_string(),
            start_time: None,
            duration: None,
        });
    }

    pub fn update_task(&self, name: &str, status: BuildStatus, progress: f64, message: String) {
        let mut tasks = self.tasks.lock();
        if let Some(task) = tasks.get_mut(name) {
            task.status = status;
            task.progress = progress;
            task.message = message;
            
            if status == BuildStatus::Running && task.start_time.is_none() {
                task.start_time = Some(Instant::now());
            }
            
            if status == BuildStatus::Completed {
                task.duration = task.start_time.map(|t| t.elapsed());
                self.completed_tasks.fetch_add(1, Ordering::SeqCst);
            }
            
            if status == BuildStatus::Failed {
                task.duration = task.start_time.map(|t| t.elapsed());
                self.failed_tasks.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    pub fn get_progress(&self) -> BuildProgress {
        let total = self.total_tasks.load(Ordering::SeqCst);
        let completed = self.completed_tasks.load(Ordering::SeqCst);
        let failed = self.failed_tasks.load(Ordering::SeqCst);
        let progress = if total > 0 {
            (completed + failed) as f64 / total as f64
        } else {
            0.0
        };

        let elapsed = self.start_time.elapsed();
        let tasks = self.tasks.lock();
        let task_list: Vec<_> = tasks.values().cloned().collect();

        BuildProgress {
            total_tasks: total,
            completed_tasks: completed,
            failed_tasks: failed,
            progress,
            elapsed,
            tasks: task_list,
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
pub struct BuildProgress {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub progress: f64,
    pub elapsed: Duration,
    pub tasks: Vec<TaskProgress>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_monitor_creation() {
        let monitor = BuildMonitor::new();
        let progress = monitor.get_progress();
        assert_eq!(progress.total_tasks, 0);
        assert_eq!(progress.progress, 0.0);
    }

    #[test]
    fn test_task_addition() {
        let monitor = BuildMonitor::new();
        monitor.add_task("test_task".to_string());
        
        let progress = monitor.get_progress();
        assert_eq!(progress.total_tasks, 1);
        assert_eq!(progress.tasks.len(), 1);
    }

    #[test]
    fn test_task_updates() {
        let monitor = BuildMonitor::new();
        monitor.add_task("test_task".to_string());
        monitor.update_task("test_task", BuildStatus::Running, 0.5, "Processing".to_string());
        
        let progress = monitor.get_progress();
        assert_eq!(progress.tasks[0].status, BuildStatus::Running);
        assert_eq!(progress.tasks[0].progress, 0.5);
    }

    #[test]
    fn test_completion_tracking() {
        let monitor = BuildMonitor::new();
        monitor.add_task("task1".to_string());
        monitor.add_task("task2".to_string());
        
        monitor.update_task("task1", BuildStatus::Completed, 1.0, "Done".to_string());
        monitor.update_task("task2", BuildStatus::Completed, 1.0, "Done".to_string());
        
        let progress = monitor.get_progress();
        assert_eq!(progress.completed_tasks, 2);
        assert_eq!(progress.progress, 1.0);
    }
}
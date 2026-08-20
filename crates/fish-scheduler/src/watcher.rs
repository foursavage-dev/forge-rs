use anyhow::{Context, Result};
use dashmap::DashSet;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub struct FsWatcherDaemon {
    watched_paths: Vec<PathBuf>,
    invalidated_targets: Arc<DashSet<String>>,
    is_running: Arc<AtomicBool>,
    debounce_duration: Duration,
    _watcher: Option<RecommendedWatcher>,
}

impl FsWatcherDaemon {
    pub fn new(debounce_duration: Duration) -> Self {
        Self {
            watched_paths: Vec::new(),
            invalidated_targets: Arc::new(DashSet::new()),
            is_running: Arc::new(AtomicBool::new(false)),
            debounce_duration,
            _watcher: None,
        }
    }

    pub fn debounce_duration(&self) -> Duration {
        self.debounce_duration
    }

    pub fn add_watch_path<P: AsRef<Path>>(&mut self, path: P) {
        self.watched_paths.push(path.as_ref().to_path_buf());
    }

    pub fn start(&mut self) -> Result<()> {
        let invalidated = Arc::clone(&self.invalidated_targets);
        let is_running = Arc::clone(&self.is_running);
        is_running.store(true, Ordering::SeqCst);

        let event_handler = move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                        for path in event.paths {
                            let path_str = path.to_string_lossy().to_string();
                            if !path_str.contains(".git") && !path_str.contains("target") {
                                invalidated.insert(path_str);
                            }
                        }
                    }
                    _ => {}
                }
            }
        };

        let mut watcher = RecommendedWatcher::new(event_handler, Config::default())
            .context("Failed to initialize filesystem watcher")?;

        for path in &self.watched_paths {
            if path.exists() {
                let _ = watcher.watch(path, RecursiveMode::Recursive);
            }
        }

        self._watcher = Some(watcher);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        self._watcher = None;
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn take_invalidated_targets(&self) -> Vec<String> {
        let mut list = Vec::new();
        let keys: Vec<String> = self
            .invalidated_targets
            .iter()
            .map(|item| item.clone())
            .collect();
        for key in keys {
            if self.invalidated_targets.remove(&key).is_some() {
                list.push(key);
            }
        }
        list
    }

    pub fn mark_dirty(&self, target: &str) {
        self.invalidated_targets.insert(target.to_string());
    }

    pub fn is_dirty(&self, target: &str) -> bool {
        self.invalidated_targets.contains(target)
    }

    pub fn mark_clean(&self, target: &str) {
        self.invalidated_targets.remove(target);
    }

    pub fn dirty_count(&self) -> usize {
        self.invalidated_targets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_watcher_daemon_dirty_tracking() {
        let watcher = FsWatcherDaemon::new(Duration::from_millis(50));
        assert_eq!(watcher.dirty_count(), 0);

        watcher.mark_dirty("crates/fish-core/src/lib.rs");
        watcher.mark_dirty("crates/fish-cli/src/main.rs");

        assert_eq!(watcher.dirty_count(), 2);
        assert!(watcher.is_dirty("crates/fish-core/src/lib.rs"));

        watcher.mark_clean("crates/fish-core/src/lib.rs");
        assert_eq!(watcher.dirty_count(), 1);
        assert!(!watcher.is_dirty("crates/fish-core/src/lib.rs"));

        let remaining = watcher.take_invalidated_targets();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0], "crates/fish-cli/src/main.rs");
        assert_eq!(watcher.dirty_count(), 0);
    }

    #[test]
    fn test_fs_watcher_lifecycle() {
        let temp = tempfile::tempdir().unwrap();
        let mut watcher = FsWatcherDaemon::new(Duration::from_millis(50));
        watcher.add_watch_path(temp.path());

        assert!(!watcher.is_running());
        watcher.start().unwrap();
        assert!(watcher.is_running());
        watcher.stop();
        assert!(!watcher.is_running());
    }
}

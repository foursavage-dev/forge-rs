//! Lifecycle states of build tasks.

/// Lifecycle state of a task in the build graph.
///
/// Transitions are not enforced by [`crate::BuildGraph`] itself; the
/// scheduler is responsible for moving tasks through their lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskState {
    /// Created, but not all dependencies have finished successfully.
    Pending,
    /// All dependencies finished successfully; waiting to be executed.
    Ready,
    /// Currently executing.
    Running,
    /// Finished successfully.
    Succeeded,
    /// Finished with a failure.
    Failed,
    /// Deliberately skipped (e.g. excluded from the requested selection).
    Skipped,
    /// Restored from cache without executing.
    Cached,
    /// Cancelled because a dependency failed or the build was aborted.
    Cancelled,
}

impl TaskState {
    /// Whether the state is terminal, meaning no further transitions are
    /// expected.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            TaskState::Succeeded
                | TaskState::Failed
                | TaskState::Skipped
                | TaskState::Cached
                | TaskState::Cancelled
        )
    }

    /// Whether the state counts as success for dependency gating: a node
    /// whose dependencies are all in a successful state is allowed to run.
    pub fn is_successful(self) -> bool {
        matches!(
            self,
            TaskState::Succeeded | TaskState::Skipped | TaskState::Cached
        )
    }

    /// Whether the state represents an unsuccessful outcome.
    pub fn is_unsuccessful(self) -> bool {
        matches!(self, TaskState::Failed | TaskState::Cancelled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_states_are_recognized() {
        for state in [
            TaskState::Succeeded,
            TaskState::Failed,
            TaskState::Skipped,
            TaskState::Cached,
            TaskState::Cancelled,
        ] {
            assert!(state.is_terminal(), "{state:?} must be terminal");
        }
        for state in [TaskState::Pending, TaskState::Ready, TaskState::Running] {
            assert!(!state.is_terminal(), "{state:?} must not be terminal");
        }
    }

    #[test]
    fn successful_states_gate_dependencies() {
        assert!(TaskState::Succeeded.is_successful());
        assert!(TaskState::Skipped.is_successful());
        assert!(TaskState::Cached.is_successful());
        assert!(!TaskState::Failed.is_successful());
        assert!(!TaskState::Cancelled.is_successful());
        assert!(!TaskState::Pending.is_successful());
        assert!(!TaskState::Ready.is_successful());
        assert!(!TaskState::Running.is_successful());
    }

    #[test]
    fn unsuccessful_states_are_exactly_failed_and_cancelled() {
        assert!(TaskState::Failed.is_unsuccessful());
        assert!(TaskState::Cancelled.is_unsuccessful());
        assert!(!TaskState::Succeeded.is_unsuccessful());
        assert!(!TaskState::Pending.is_unsuccessful());
    }
}

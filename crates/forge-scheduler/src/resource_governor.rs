use sysinfo::System;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPressureLevel {
    Normal,
    Warning,
    Critical,
}

pub struct KernelResourceGovernor {
    max_ram_bytes: Option<u64>,
    ram_limit_pct: u8,
}

impl KernelResourceGovernor {
    pub fn new(max_ram_bytes: Option<u64>, ram_limit_pct: Option<u8>) -> Self {
        Self {
            max_ram_bytes,
            ram_limit_pct: ram_limit_pct.unwrap_or(85),
        }
    }

    pub fn check_memory_pressure(&self) -> MemoryPressureLevel {
        let mut sys = System::new();
        sys.refresh_memory();

        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();

        if total_mem == 0 {
            return MemoryPressureLevel::Normal;
        }

        let used_pct = ((used_mem as f64 / total_mem as f64) * 100.0) as u8;

        if let Some(limit_bytes) = self.max_ram_bytes
            && used_mem >= limit_bytes
        {
            return MemoryPressureLevel::Critical;
        }

        if used_pct >= self.ram_limit_pct {
            MemoryPressureLevel::Critical
        } else if used_pct >= self.ram_limit_pct.saturating_sub(10) {
            MemoryPressureLevel::Warning
        } else {
            MemoryPressureLevel::Normal
        }
    }

    pub fn should_throttle(&self) -> bool {
        self.check_memory_pressure() == MemoryPressureLevel::Critical
    }

    pub fn optimal_parallelism(&self, base_jobs: usize) -> usize {
        match self.check_memory_pressure() {
            MemoryPressureLevel::Normal => base_jobs.max(1),
            MemoryPressureLevel::Warning => (base_jobs * 3 / 4).max(1),
            MemoryPressureLevel::Critical => (base_jobs / 2).max(1),
        }
    }
}

impl Default for KernelResourceGovernor {
    fn default() -> Self {
        Self::new(None, Some(85))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_governor_defaults() {
        let governor = KernelResourceGovernor::default();
        let level = governor.check_memory_pressure();
        assert!(matches!(
            level,
            MemoryPressureLevel::Normal
                | MemoryPressureLevel::Warning
                | MemoryPressureLevel::Critical
        ));
    }

    #[test]
    fn test_optimal_parallelism_calculation() {
        let governor = KernelResourceGovernor::new(None, Some(99));
        let jobs = governor.optimal_parallelism(8);
        assert!((1..=8).contains(&jobs));
    }
}

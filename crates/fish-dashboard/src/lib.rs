#![forbid(unsafe_code)]

pub mod api;
pub mod dashboard;
pub mod flamegraph;
pub mod metrics;

pub use dashboard::DashboardServer;
pub use flamegraph::FlamegraphGenerator;
pub use metrics::BuildMetrics;

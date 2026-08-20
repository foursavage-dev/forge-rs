use std::path::{Path, PathBuf};

use fish_backend_rust::BuildMode;
use fish_cache::LocalCache;
use fish_scheduler::Scheduler;

use crate::args::CommonArgs;
use crate::render;

/// Resolves the start directory for the build, handling optional path input.
pub fn resolve_start_dir(path: Option<&Path>) -> Result<PathBuf, String> {
    let base = match path {
        Some(path) => {
            if path.is_file() {
                return Err(format!(
                    "`{}` is a file; expected a project directory",
                    path.display()
                ));
            }
            path.to_path_buf()
        }
        None => std::env::current_dir()
            .map_err(|error| format!("failed to determine the current directory: {error}"))?,
    };
    std::fs::canonicalize(&base)
        .map_err(|error| format!("cannot access `{}`: {error}", base.display()))
}

/// Returns a plain path representation, stripping Windows UNC prefix if present.
pub fn plain_path(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    if cfg!(windows)
        && let Some(stripped) = text.strip_prefix(r"\\?\")
    {
        return PathBuf::from(stripped);
    }
    path.to_path_buf()
}

/// Returns the default number of parallel jobs based on available CPU cores.
pub fn default_jobs() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2)
}

/// Returns a verb describing the build mode (e.g., "Building", "Checking", "Testing").
pub fn mode_verb(mode: BuildMode) -> &'static str {
    match mode {
        BuildMode::Build => "Building",
        BuildMode::Check => "Checking",
        BuildMode::Test => "Testing",
    }
}

/// Opens the local fingerprint cache, honouring `--cache-dir` and `--no-cache`.
pub fn open_cache(args: &CommonArgs) -> Option<LocalCache> {
    if args.no_cache {
        return None;
    }
    let result = match &args.cache_dir {
        Some(dir) => LocalCache::new(dir),
        None => LocalCache::default_location(),
    };
    match result {
        Ok(cache) => {
            if !args.tui {
                render::print_cache_location(cache.root());
            }
            Some(cache)
        }
        Err(error) => {
            if !args.tui {
                eprintln!("warning: fingerprint cache disabled: {error}");
            }
            None
        }
    }
}

/// Builds the scheduler, enabling RAM backpressure when `--ram-limit` is
/// given. The throttled worker floor is `jobs / 2` (minimum 1).
pub fn make_scheduler(workers: usize, args: &CommonArgs) -> Scheduler {
    let mut scheduler = Scheduler::new(workers);
    if let Some(limit) = args.ram_limit {
        let floor = (workers / 2).max(1);
        scheduler = scheduler.with_ram_backpressure(limit, floor);
    }
    scheduler
}

/// Formats a byte count as a human-readable string (e.g., "1.5 GB").
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Parses durations like `30m`, `2h`, `7d` (bare numbers are seconds).
pub fn parse_duration(text: &str) -> Result<std::time::Duration, String> {
    let text = text.trim();
    let (num, unit) = match text.chars().last() {
        Some(c) if c.is_ascii_digit() => (text, ""),
        _ => (&text[..text.len() - 1], &text[text.len() - 1..]),
    };
    let value: u64 = num
        .trim()
        .parse()
        .map_err(|_| format!("invalid duration `{text}` (expected e.g. `7d`, `12h`, `30m`)"))?;
    let seconds = match unit {
        "" | "s" => value,
        "m" => value * 60,
        "h" => value * 3600,
        "d" => value * 86400,
        _ => {
            return Err(format!(
                "unknown duration unit `{unit}` in `{text}` (expected s, m, h, or d)"
            ));
        }
    };
    Ok(std::time::Duration::from_secs(seconds))
}

/// Parses sizes like `500MB`, `2GB`, `10KB` (binary multiples).
pub fn parse_size(text: &str) -> Result<u64, String> {
    let mut text = text.trim().to_uppercase();
    if text.ends_with('B') {
        text.pop();
    }
    let (num, unit) = match text.chars().last() {
        Some(c) if c.is_ascii_digit() => (text.as_str(), "B"),
        _ => (&text[..text.len() - 1], &text[text.len() - 1..]),
    };
    let value: u64 = num
        .trim()
        .parse()
        .map_err(|_| format!("invalid size `{text}` (expected e.g. `10GB`, `500MB`)"))?;
    let factor = match unit {
        "B" => 1u64,
        "K" => 1024,
        "M" => 1024 * 1024,
        "G" => 1024 * 1024 * 1024,
        _ => {
            return Err(format!(
                "unknown size unit `{unit}` in `{text}` (expected B, KB, MB, or GB)"
            ));
        }
    };
    Ok(value * factor)
}

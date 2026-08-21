use std::io::{self, Stdout};
use std::time::Instant;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

use fish_executor::TaskOutcome;
use fish_scheduler::BuildSummary;

#[derive(Debug, Clone)]
pub struct TuiTaskEntry {
    pub label: String,
    pub status: String,
    pub duration_ms: u64,
}

pub struct TuiDashboard {
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
    start_time: Instant,
    tasks: Vec<TuiTaskEntry>,
    logs: Vec<String>,
    total_tasks: usize,
    completed: usize,
    cpu_history: Vec<u64>,
    mem_history: Vec<u64>,
    #[cfg(target_os = "linux")]
    last_sys: Option<SysSnapshot>,
}

impl TuiDashboard {
    pub fn new(total_tasks: usize) -> Self {
        Self {
            terminal: None,
            start_time: Instant::now(),
            tasks: Vec::new(),
            logs: Vec::new(),
            total_tasks,
            completed: 0,
            cpu_history: Vec::new(),
            mem_history: Vec::new(),
            #[cfg(target_os = "linux")]
            last_sys: None,
        }
    }

    pub fn start(&mut self) -> Result<(), io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        self.terminal = Some(terminal);
        self.draw()?;
        Ok(())
    }

    pub fn on_task_finish(&mut self, label: &str, outcome: &TaskOutcome) {
        self.completed += 1;
        let status = format!("{:?}", outcome.status);
        let duration_ms = outcome.duration.as_millis() as u64;
        self.tasks.push(TuiTaskEntry {
            label: label.to_string(),
            status,
            duration_ms,
        });

        if !outcome.stdout.trim().is_empty() {
            for line in outcome.stdout.lines() {
                self.logs.push(format!("[{label}] {line}"));
            }
        }
        if !outcome.stderr.trim().is_empty() {
            for line in outcome.stderr.lines() {
                self.logs.push(format!("[{label} ERR] {line}"));
            }
        }

        let _ = self.draw();
    }

    pub fn draw(&mut self) -> Result<(), io::Error> {
        if let Some(ref mut terminal) = self.terminal {
            let total = self.total_tasks.max(1);
            let completed = self.completed;
            let ratio = (completed as f64 / total as f64).clamp(0.0, 1.0);
            let elapsed = self.start_time.elapsed().as_secs_f64();
            let tasks = self.tasks.clone();
            let logs = self.logs.clone();

            // Sample system utilization once per frame (Linux only, via /proc).
            #[cfg(target_os = "linux")]
            {
                if let Some(cur) = read_sys_snapshot() {
                    if let Some(prev) = self.last_sys {
                        self.cpu_history.push(cpu_percent(&prev, &cur));
                        if self.cpu_history.len() > 80 {
                            self.cpu_history.remove(0);
                        }
                        let mem_pct = (100 * cur.mem_used_kb / cur.mem_total_kb.max(1)).min(100);
                        self.mem_history.push(mem_pct);
                        if self.mem_history.len() > 80 {
                            self.mem_history.remove(0);
                        }
                    }
                    self.last_sys = Some(cur);
                }
            }
            let cpu_hist = self.cpu_history.clone();
            let mem_hist = self.mem_history.clone();

            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(4),
                        Constraint::Min(5),
                        Constraint::Length(1),
                    ])
                    .split(f.area());

                let header_text = vec![Line::from(vec![
                    Span::styled(
                        "FORGE BUILD DASHBOARD ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("(Elapsed: {elapsed:.2}s, Tasks: {completed}/{total})"),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])];
                let header = Paragraph::new(header_text)
                    .block(Block::default().borders(Borders::ALL).title("Status"));
                f.render_widget(header, chunks[0]);

                let gauge = Gauge::default()
                    .block(Block::default().borders(Borders::ALL).title("Progress"))
                    .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
                    .percent((ratio * 100.0) as u16)
                    .label(format!("{completed}/{total} ({:.0}%)", ratio * 100.0));
                f.render_widget(gauge, chunks[1]);

                let cpu_last = cpu_hist.last().copied().unwrap_or(0);
                let mem_last = mem_hist.last().copied().unwrap_or(0);
                let resources = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled(
                            "CPU ",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!("{cpu_last:>3}% ")),
                        Span::styled(
                            sparkline(&cpu_hist, 60, 100),
                            Style::default().fg(Color::Cyan),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "MEM ",
                            Style::default()
                                .fg(Color::Magenta)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!("{mem_last:>3}% ")),
                        Span::styled(
                            sparkline(&mem_hist, 60, 100),
                            Style::default().fg(Color::Magenta),
                        ),
                    ]),
                ])
                .block(Block::default().borders(Borders::ALL).title("CPU / RAM"));
                f.render_widget(resources, chunks[2]);

                let body_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                    .split(chunks[3]);

                let task_items: Vec<ListItem> = tasks
                    .iter()
                    .rev()
                    .take(20)
                    .map(|t| {
                        let color =
                            if t.status.contains("Success") || t.status.contains("SkippedCached") {
                                Color::Green
                            } else {
                                Color::Red
                            };
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("{:<25} ", t.label),
                                Style::default().fg(Color::White),
                            ),
                            Span::styled(format!("[{}] ", t.status), Style::default().fg(color)),
                            Span::styled(
                                format!("{}ms", t.duration_ms),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]))
                    })
                    .collect();

                let task_list = List::new(task_items)
                    .block(Block::default().borders(Borders::ALL).title("Recent Tasks"));
                f.render_widget(task_list, body_chunks[0]);

                let log_items: Vec<ListItem> = logs
                    .iter()
                    .rev()
                    .take(20)
                    .map(|l| ListItem::new(Span::raw(l.clone())))
                    .collect();
                let log_list = List::new(log_items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Execution Logs"),
                );
                f.render_widget(log_list, body_chunks[1]);

                let footer = Paragraph::new("Press 'Ctrl+C' to cancel build")
                    .style(Style::default().fg(Color::DarkGray));
                f.render_widget(footer, chunks[4]);
            })?;
        }
        Ok(())
    }

    pub fn finish(&mut self, summary: &BuildSummary) -> Result<(), io::Error> {
        // Render a final frame with the per-task waterfall timeline before
        // leaving the alternate screen.
        if let Some(ref mut terminal) = self.terminal {
            let rows: Vec<(String, u64, u64)> = summary
                .timings
                .iter()
                .map(|t| {
                    (
                        t.label.clone(),
                        t.start_offset.as_millis() as u64,
                        t.duration.as_millis() as u64,
                    )
                })
                .collect();
            let _ = terminal.draw(|f| {
                let area = f.area();
                let lines = waterfall_lines(&rows, area.width as usize);
                let waterfall = Paragraph::new(lines.join("\n")).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Task Waterfall (start → end)"),
                );
                f.render_widget(waterfall, area);
            });
        }

        if self.terminal.is_some() {
            disable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, LeaveAlternateScreen)?;
        }
        self.terminal = None;
        Ok(())
    }
}

impl Drop for TuiDashboard {
    fn drop(&mut self) {
        if self.terminal.is_some() {
            let _ = disable_raw_mode();
            let mut stdout = io::stdout();
            let _ = execute!(stdout, LeaveAlternateScreen);
        }
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy)]
struct SysSnapshot {
    cpu_idle: u64,
    cpu_total: u64,
    mem_used_kb: u64,
    mem_total_kb: u64,
}

/// Read aggregate CPU counters and memory usage from `/proc`. Returns `None`
/// when the procfs files are unavailable (e.g. inside a restricted sandbox),
/// in which case the dashboard simply keeps the previous graphs empty.
#[cfg(target_os = "linux")]
fn read_sys_snapshot() -> Option<SysSnapshot> {
    let stat = std::fs::read_to_string("/proc/stat").ok()?;
    let cpu_line = stat.lines().next()?;
    let fields: Vec<u64> = cpu_line
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    // Aggregate line: "cpu  user nice system idle iowait irq softirq steal ..."
    let idle = fields.get(3).copied().unwrap_or(0) + fields.get(4).copied().unwrap_or(0);
    let total: u64 = fields.iter().sum();

    let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut mem_total = 0u64;
    let mut mem_available = 0u64;
    for line in meminfo.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            mem_total = parse_kib(rest);
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            mem_available = parse_kib(rest);
        }
    }
    if mem_total == 0 {
        return None;
    }
    Some(SysSnapshot {
        cpu_idle: idle,
        cpu_total: total,
        mem_used_kb: mem_total.saturating_sub(mem_available),
        mem_total_kb: mem_total,
    })
}

/// Extract the numeric value from a `/proc/meminfo` field such as `"16000000 kB"`.
#[cfg(target_os = "linux")]
fn parse_kib(field: &str) -> u64 {
    field
        .split_whitespace()
        .next()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

/// Percentage of CPU time spent outside idle between two snapshots, clamped
/// to `0..=100`.
#[cfg(target_os = "linux")]
fn cpu_percent(prev: &SysSnapshot, cur: &SysSnapshot) -> u64 {
    let total_delta = cur.cpu_total.saturating_sub(prev.cpu_total);
    let idle_delta = cur.cpu_idle.saturating_sub(prev.cpu_idle);
    (100 * (total_delta - idle_delta))
        .checked_div(total_delta)
        .unwrap_or(0)
        .min(100)
}

/// Render a sparkline of the most recent `width` samples using Unicode block
/// glyphs. Values are normalized against `max` (which must be non-zero).
fn sparkline(values: &[u64], width: usize, max: u64) -> String {
    const GLYPHS: &[char] = &[
        '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
        '\u{2588}',
    ];
    if values.is_empty() || width == 0 {
        return String::new();
    }
    let max = max.max(1);
    let tail: Vec<u64> = values.iter().rev().take(width).copied().rev().collect();
    let mut out = String::with_capacity(tail.len());
    for value in tail {
        let ratio = (value as f64 / max as f64).clamp(0.0, 1.0);
        let index = (ratio * (GLYPHS.len() - 1) as f64) as usize;
        out.push(GLYPHS[index]);
    }
    out
}

/// Render a Gantt-style waterfall: one line per task with a label column and a
/// timeline of filled blocks positioned by start offset and sized by duration.
/// Returns text lines ready for a `Paragraph` widget.
fn waterfall_lines(rows: &[(String, u64, u64)], width: usize) -> Vec<String> {
    const LABEL_WIDTH: usize = 24;
    if rows.is_empty() {
        return vec!["(no task timings recorded)".to_string()];
    }

    let timeline_width = width.saturating_sub(LABEL_WIDTH + 3).max(12);
    // Normalize against the schedule end so the widest bar spans the timeline.
    let total_ms = rows
        .iter()
        .map(|(_, start, dur)| start.saturating_add(*dur))
        .max()
        .unwrap_or(1)
        .max(1);

    let mut lines = Vec::with_capacity(rows.len());
    for (label, start, dur) in rows {
        let start_col = scale(*start, total_ms, timeline_width);
        let bar_len = scale(*dur, total_ms, timeline_width).max(1);
        let mut line = format!("{:<label_width$} │", label, label_width = LABEL_WIDTH);
        for col in 0..timeline_width {
            if col >= start_col && col < start_col + bar_len {
                line.push('█');
            } else {
                line.push(' ');
            }
        }
        lines.push(line);
    }
    lines
}

/// Scale `value` into `[0, width]` proportionally to `total`, saturating at
/// `width`.
fn scale(value: u64, total: u64, width: usize) -> usize {
    if total == 0 {
        return 0;
    }
    ((value as u128 * width as u128) / total as u128).min(width as u128) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparkline_renders_extremes() {
        assert_eq!(sparkline(&[], 10, 100), "");
        assert_eq!(sparkline(&[100], 10, 100), "█");
        assert_eq!(sparkline(&[0], 10, 100), "▁");
    }

    #[test]
    fn sparkline_shows_only_the_most_recent_samples() {
        let line = sparkline(&[0, 25, 50, 75, 100], 3, 100);
        assert_eq!(line.chars().count(), 3, "only the last 3 samples are drawn");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn cpu_percent_detects_busy_and_idle_deltas() {
        let zero = SysSnapshot {
            cpu_idle: 0,
            cpu_total: 0,
            mem_used_kb: 0,
            mem_total_kb: 1,
        };
        let busy = SysSnapshot {
            cpu_idle: 0,
            cpu_total: 100,
            mem_used_kb: 0,
            mem_total_kb: 1,
        };
        assert_eq!(cpu_percent(&zero, &busy), 100);

        let idle_prev = SysSnapshot {
            cpu_idle: 50,
            cpu_total: 50,
            mem_used_kb: 0,
            mem_total_kb: 1,
        };
        let idle_cur = SysSnapshot {
            cpu_idle: 100,
            cpu_total: 100,
            mem_used_kb: 0,
            mem_total_kb: 1,
        };
        assert_eq!(cpu_percent(&idle_prev, &idle_cur), 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parse_kib_extracts_numeric_value() {
        assert_eq!(parse_kib("16000000 kB"), 16_000_000);
    }

    #[test]
    fn waterfall_renders_one_line_per_task() {
        let rows = vec![
            ("compile core".to_string(), 0, 100),
            ("compile cli".to_string(), 100, 200),
        ];
        let lines = waterfall_lines(&rows, 80);
        assert_eq!(lines.len(), 2);
        // The second task starts at 100/300 of the timeline and spans 200/300.
        assert!(lines[0].contains('█'));
        assert!(lines[1].contains('█'));
    }

    #[test]
    fn waterfall_handles_empty_and_zero_width_inputs() {
        assert!(waterfall_lines(&[], 80).iter().all(|l| !l.is_empty()));
        let rows = vec![("task".to_string(), 0, 0)];
        let lines = waterfall_lines(&rows, 10);
        // Zero-duration tasks still render a one-block sliver.
        assert!(lines[0].contains('█'));
    }

    #[test]
    fn scale_is_proportional_and_saturating() {
        assert_eq!(scale(0, 100, 50), 0);
        assert_eq!(scale(50, 100, 50), 25);
        assert_eq!(scale(200, 100, 50), 50, "must saturate at the width");
    }
}

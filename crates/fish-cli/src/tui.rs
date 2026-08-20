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

            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
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

                let body_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                    .split(chunks[2]);

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
                f.render_widget(footer, chunks[3]);
            })?;
        }
        Ok(())
    }

    pub fn finish(&mut self, _summary: &BuildSummary) -> Result<(), io::Error> {
        let _ = self.draw();
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

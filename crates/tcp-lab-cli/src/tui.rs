use std::{
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Chart, Axis, Dataset, GraphType},
};
use tcp_lab_core::Simulator;

/// A tracing subscriber that writes to a shared buffer for TUI display
#[derive(Clone)]
pub struct MemoryLogBuffer {
    logs: Arc<Mutex<Vec<String>>>,
}

impl MemoryLogBuffer {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn push(&self, msg: String) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(msg);
        // Keep last 1000 logs
        if logs.len() > 1000 {
            logs.remove(0);
        }
    }
    
    /// Return a clone of all log lines (for scrolling computations in TUI)
    pub fn all_lines(&self) -> Vec<String> {
        let logs = self.logs.lock().unwrap();
        logs.clone()
    }
}

impl io::Write for MemoryLogBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        // tracing-subscriber adds newlines, we might want to trim them or keep them
        self.push(s.trim().to_string());
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct TuiApp {
    simulator: Simulator,
    log_buffer: MemoryLogBuffer,
    paused: bool,
    scenario_name: Option<String>,
    /// How many lines from the bottom we are scrolled up in the log view
    log_scroll: usize,
}

impl TuiApp {
    pub fn new(simulator: Simulator, log_buffer: MemoryLogBuffer, scenario_name: Option<String>) -> Self {
        Self {
            simulator,
            log_buffer,
            paused: true, // Start paused
            scenario_name,
            log_scroll: 0,
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();

        // Init sim
        self.simulator.init();

        loop {
            terminal.draw(|f| self.ui(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char(' ') => self.paused = !self.paused,
                        KeyCode::Char('s') => { // Step once
                            self.simulator.step();
                        },
                        KeyCode::Up => {
                            self.log_scroll = self.log_scroll.saturating_add(1);
                        },
                        KeyCode::Down => {
                            if self.log_scroll > 0 {
                                self.log_scroll -= 1;
                            }
                        },
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if !self.paused {
                    // Advance simulation
                    // We can do multiple steps per frame if needed
                    if self.simulator.step() {
                        // Continue
                    } else {
                        // Simulation finished
                        self.paused = true;
                    }
                }
                last_tick = Instant::now();
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn ui(&self, f: &mut Frame) {
        // Root layout: main view (top) + logs (bottom)
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(10),
            ])
            .split(f.area());

        let main_area = root[0];
        let log_area = root[1];

        // Main area: left (dashboard) + right (window history / extra views)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(main_area);

        // Left: Dashboard
        self.render_dashboard(f, main_chunks[0]);

        // Right: Window history / additional view
        self.render_window_history(f, main_chunks[1]);

        // Bottom: Logs (full-width, scrollable)
        self.render_logs(f, log_area);
    }

    fn render_dashboard(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Status bar
                Constraint::Min(0),    // Main stats
            ])
            .split(area);

        let scenario = self
            .scenario_name
            .as_deref()
            .unwrap_or("Ad-hoc Simulation");
        let status_text = format!(
            "Scenario: {} | Time: {} ms | Status: {} | Events Pending: {} | (q)uit (space)pause/resume (s)tep",
            scenario,
            self.simulator.current_time(),
            if self.paused { "PAUSED" } else { "RUNNING" },
            self.simulator.remaining_events()
        );

        let status_block = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title("Control"));
        f.render_widget(status_block, chunks[0]);
        
        // Stats
        let delivered = self.simulator.delivered_data.len();
        let sent_packets = self.simulator.sender_packet_count;
        let (win_current, win_max) = if self.simulator.sender_window_sizes.is_empty() {
            (0u16, 0u16)
        } else {
            let max = *self.simulator.sender_window_sizes.iter().max().unwrap_or(&0);
            let cur = *self
                .simulator
                .sender_window_sizes
                .last()
                .unwrap_or(&0);
            (cur, max)
        };

        let cfg = self.simulator.config();
        let stats_text = vec![
            Line::from("Simulation Stats:"),
            Line::from(format!("  Delivered messages: {}", delivered)),
            Line::from(format!("  Sender packets:     {}", sent_packets)),
            Line::from(format!(
                "  Sender window:      current={} max={}",
                win_current, win_max
            )),
            Line::from(format!(
                "  Channel: loss={:.2}, corrupt={:.2}, latency={}..{} ms",
                cfg.loss_rate, cfg.corrupt_rate, cfg.min_latency, cfg.max_latency
            )),
            Line::from(""),
            Line::from("Controls:"),
            Line::from("  Space: Pause/Resume"),
            Line::from("  s:     Step one event"),
            Line::from("  q:     Quit"),
        ];
        
        let stats_block = Paragraph::new(stats_text)
            .block(Block::default().borders(Borders::ALL).title("Dashboard"));
        f.render_widget(stats_block, chunks[1]);
    }

    fn render_window_history(&self, f: &mut Frame, area: Rect) {
        // 构造一张叠加图：前景 cwnd，背景 ssthresh
        // cwnd 优先来自 metrics("cwnd")，否则退化为 sender_window_sizes（按采样顺序）

        let mut x_min = f64::MAX;
        let mut x_max = f64::MIN;
        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;

        let mut update_bounds = |pts: &[(f64, f64)]| {
            for (x, y) in pts {
                if *x < x_min { x_min = *x; }
                if *x > x_max { x_max = *x; }
                if *y < y_min { y_min = *y; }
                if *y > y_max { y_max = *y; }
            }
        };

        // 先收集所有序列，避免同时对 Vec 可变+不可变借用
        let mut cwnd_series_vec: Option<Vec<(f64, f64)>> = None;
        let mut ssthresh_series_vec: Option<Vec<(f64, f64)>> = None;

        // cwnd 系列
        if let Some(cwnd_series) = self.simulator.metric_series("cwnd") {
            if !cwnd_series.is_empty() {
                let pts: Vec<(f64, f64)> = cwnd_series
                    .iter()
                    .map(|(t, v)| (*t as f64, *v))
                    .collect();
                update_bounds(&pts);
                cwnd_series_vec = Some(pts);
            }
        } else if !self.simulator.sender_window_sizes.is_empty() {
            let pts: Vec<(f64, f64)> = self
                .simulator
                .sender_window_sizes
                .iter()
                .enumerate()
                .map(|(i, w)| (i as f64, *w as f64))
                .collect();
            update_bounds(&pts);
            cwnd_series_vec = Some(pts);
        }

        // ssthresh 系列（只有 Reno/Tahoe 会报）
        if let Some(series) = self.simulator.metric_series("ssthresh") {
            if !series.is_empty() {
                let pts: Vec<(f64, f64)> = series
                    .iter()
                    .map(|(t, v)| (*t as f64, *v))
                    .collect();
                update_bounds(&pts);
                ssthresh_series_vec = Some(pts);
            }
        }

        let mut datasets: Vec<Dataset> = Vec::new();

        if let Some(ref pts) = cwnd_series_vec {
            datasets.push(
                Dataset::default()
                    .name("cwnd")
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Cyan))
                    .graph_type(GraphType::Line)
                    .data(pts),
            );
        }

        if let Some(ref pts) = ssthresh_series_vec {
            datasets.push(
                Dataset::default()
                    .name("ssthresh")
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::DIM))
                    .graph_type(GraphType::Line)
                    .data(pts),
            );
        }

        if datasets.is_empty() {
            let block = Paragraph::new("No window metrics yet")
                .block(Block::default().borders(Borders::ALL).title("Window"));
            f.render_widget(block, area);
            return;
        }

        if x_min == f64::MAX {
            x_min = 0.0;
            x_max = 1.0;
            y_min = 0.0;
            y_max = 1.0;
        } else {
            if (x_max - x_min).abs() < f64::EPSILON {
                x_max += 1.0;
            }
            if (y_max - y_min).abs() < f64::EPSILON {
                y_max += 1.0;
            }
        }

        let x_labels = vec![
            Span::raw(format!("{:.0}", x_min)),
            Span::raw(""),
            Span::raw(format!("{:.0}", x_max)),
        ];
        let y_labels = vec![
            Span::raw(format!("{:.0}", y_min)),
            Span::raw(""),
            Span::raw(format!("{:.0}", y_max)),
        ];

        let chart = Chart::new(datasets)
            .block(Block::default().borders(Borders::ALL).title("Sender Window / ssthresh"))
            .x_axis(
                Axis::default()
                    .title("time")
                    .bounds([x_min, x_max])
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    .title("size")
                    .bounds([y_min, y_max])
                    .labels(y_labels),
            );

        f.render_widget(chart, area);
    }

    fn render_logs(&self, f: &mut Frame, area: Rect) {
        let height = area.height.max(3) as usize;
        let visible = height - 2; // account for borders
        let all_logs = self.log_buffer.all_lines();
        let total = all_logs.len();

        // Compute scroll bounds
        let max_scroll = total.saturating_sub(visible);
        let scroll = self.log_scroll.min(max_scroll);
        let start = total.saturating_sub(visible + scroll);
        let end = total.saturating_sub(scroll);
        let start = start.max(0);
        let end = end.max(start);
        let slice = &all_logs[start..end];
        
        let items: Vec<ListItem> = slice
            .iter()
            .map(|log| {
                // 简单按前缀为 Sender/Receiver 日志着色
                let style = if log.contains("[Sender]") {
                    Style::default().fg(Color::Cyan)
                } else if log.contains("[Receiver]") {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(log.as_str(), style)))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Logs"));
        
        f.render_widget(list, area);
    }
}

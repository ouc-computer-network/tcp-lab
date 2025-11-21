use std::{
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::engine::Simulator;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::widgets::canvas::{Canvas, Line as CanvasLine, Points};
use ratatui::{
    prelude::*,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph},
};

/// A tracing subscriber that writes to a shared buffer for TUI display
#[derive(Clone)]
pub struct MemoryLogBuffer {
    logs: Arc<Mutex<Vec<String>>>,
}

impl Default for MemoryLogBuffer {
    fn default() -> Self {
        Self::new()
    }
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
    paused: bool,
    scenario_name: Option<String>,
    /// Vertical scroll offset for link events list
    link_scroll: usize,
}

impl TuiApp {
    pub fn new(simulator: Simulator, scenario_name: Option<String>) -> Self {
        Self {
            simulator,
            paused: true, // Start paused
            scenario_name,
            link_scroll: 0,
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

            if crossterm::event::poll(timeout)?
                && let Event::Key(key) = event::read()?
            {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => self.paused = !self.paused,
                    KeyCode::Char('s') => {
                        // Step once
                        self.simulator.step();
                    }
                    // Vertical scroll in link events list
                    KeyCode::Up => {
                        self.link_scroll = self.link_scroll.saturating_add(1);
                    }
                    KeyCode::Down => {
                        if self.link_scroll > 0 {
                            self.link_scroll -= 1;
                        }
                    }
                    _ => {}
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

    pub fn into_simulator(self) -> Simulator {
        self.simulator
    }

    fn ui(&self, f: &mut Frame) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Control bar
                Constraint::Length(10), // Link space-time
                Constraint::Min(0),     // Split dashboard + window
                Constraint::Length(10), // Link events
            ])
            .split(f.area());

        self.render_control(f, rows[0]);
        self.render_link_space_time(f, rows[1]);

        let mid_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[2]);
        self.render_dashboard_body(f, mid_chunks[0]);
        self.render_window_history(f, mid_chunks[1]);

        self.render_link_events(f, rows[3]);
    }

    fn render_control(&self, f: &mut Frame, area: Rect) {
        let scenario = self.scenario_name.as_deref().unwrap_or("Ad-hoc Simulation");
        let status_text = format!(
            "Scenario: {} | Time: {} ms | Status: {} | Events Pending: {} | (q)uit (space)pause/resume (s)tep",
            scenario,
            self.simulator.current_time(),
            if self.paused { "PAUSED" } else { "RUNNING" },
            self.simulator.remaining_events()
        );
        let status_block = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title("Control"));
        f.render_widget(status_block, area);
    }

    fn render_dashboard_body(&self, f: &mut Frame, area: Rect) {
        // Stats
        let delivered = self.simulator.delivered_data.len();
        let sent_packets = self.simulator.sender_packet_count;
        let (win_current, win_max) = if self.simulator.sender_window_sizes.is_empty() {
            (0u16, 0u16)
        } else {
            let max = *self
                .simulator
                .sender_window_sizes
                .iter()
                .max()
                .unwrap_or(&0);
            let cur = *self.simulator.sender_window_sizes.last().unwrap_or(&0);
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

        // Stats block
        let stats_block = Paragraph::new(stats_text)
            .block(Block::default().borders(Borders::ALL).title("Dashboard"));
        f.render_widget(stats_block, area);
    }

    fn render_window_history(&self, f: &mut Frame, area: Rect) {
        // 构造一张叠加图：前景 cwnd，背景 ssthresh
        // cwnd 优先来自 metrics("cwnd")，否则退化为 sender_window_sizes（按采样顺序）

        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;

        // 先收集所有序列，避免同时对 Vec 可变+不可变借用
        let mut cwnd_series_vec: Option<Vec<(f64, f64)>> = None;
        let mut ssthresh_series_vec: Option<Vec<(f64, f64)>> = None;

        // cwnd 系列
        if let Some(cwnd_series) = self.simulator.metric_series("cwnd") {
            if !cwnd_series.is_empty() {
                let pts: Vec<(f64, f64)> = cwnd_series
                    .iter()
                    .enumerate()
                    .map(|(i, (_, v))| (i as f64, *v))
                    .collect();
                if !pts.is_empty() {
                    for (_, y) in &pts {
                        if *y < y_min {
                            y_min = *y;
                        }
                        if *y > y_max {
                            y_max = *y;
                        }
                    }
                    cwnd_series_vec = Some(pts);
                }
            }
        } else if !self.simulator.sender_window_sizes.is_empty() {
            // 没有 metric 时退化为按索引显示，不支持时间缩放
            let pts: Vec<(f64, f64)> = self
                .simulator
                .sender_window_sizes
                .iter()
                .enumerate()
                .map(|(i, w)| (i as f64, *w as f64))
                .collect();
            for (_, y) in &pts {
                if *y < y_min {
                    y_min = *y;
                }
                if *y > y_max {
                    y_max = *y;
                }
            }
            cwnd_series_vec = Some(pts);
        }

        // ssthresh 系列（只有 Reno/Tahoe 会报）
        if let Some(series) = self.simulator.metric_series("ssthresh")
            && !series.is_empty()
        {
            let pts: Vec<(f64, f64)> = series
                .iter()
                .enumerate()
                .map(|(i, (_, v))| (i as f64, *v))
                .collect();
            if !pts.is_empty() {
                for (_, y) in &pts {
                    if *y < y_min {
                        y_min = *y;
                    }
                    if *y > y_max {
                        y_max = *y;
                    }
                }
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
                    .style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::DIM),
                    )
                    .graph_type(GraphType::Line)
                    .data(pts),
            );
        }

        if datasets.is_empty() || y_min == f64::MAX {
            let block = Paragraph::new("No window metrics yet")
                .block(Block::default().borders(Borders::ALL).title("Window"));
            f.render_widget(block, area);
            return;
        }

        if (y_max - y_min).abs() < f64::EPSILON {
            y_max += 1.0;
        }

        let x_labels = vec![Span::raw("0"), Span::raw(""), Span::raw("n")];
        let y_labels = vec![
            Span::raw(format!("{:.0}", y_min)),
            Span::raw(""),
            Span::raw(format!("{:.0}", y_max)),
        ];

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Sender Window / ssthresh"),
            )
            .x_axis(
                Axis::default()
                    .title("time")
                    .bounds([
                        0.0,
                        cwnd_series_vec
                            .as_ref()
                            .map(|v| v.len() as f64)
                            .unwrap_or(1.0),
                    ])
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

    fn render_link_space_time(&self, f: &mut Frame, area: Rect) {
        let events = &self.simulator.link_events;
        if events.is_empty() {
            let block = Paragraph::new("No link activity yet")
                .block(Block::default().borders(Borders::ALL).title("Link"));
            f.render_widget(block, area);
            return;
        }

        // 仅展示最近若干个事件，形成简单的“局部时空图”
        let max_events = (area.width as usize).saturating_sub(4).max(4);
        let tail = events.iter().rev().take(max_events).collect::<Vec<_>>();
        let window_events: Vec<_> = tail.into_iter().rev().collect();

        let t_min = window_events.first().map(|e| e.time as f64).unwrap_or(0.0);
        let mut t_max = window_events.last().map(|e| e.time as f64).unwrap_or(1.0);
        if (t_max - t_min).abs() < f64::EPSILON {
            t_max += 1.0;
        }

        // 构造发送箭头（Sender/Receiver 之间的斜线）
        let mut lines: Vec<CanvasLine> = Vec::new();
        let mut drop_points: Vec<(f64, f64)> = Vec::new();
        let mut corrupt_points: Vec<(f64, f64)> = Vec::new();
        let mut annotations: Vec<(f64, f64, String, Color)> = Vec::new();

        for e in &window_events {
            let desc = e.description.as_str();
            let t0 = e.time as f64;
            let direction = detect_direction(desc);

            if desc.contains("SEND") {
                // 方向：Sender->Receiver 或 Receiver->Sender
                let (y_src, y_dst) = match direction {
                    LinkDirection::SenderToReceiver => (0.0, 2.0),
                    LinkDirection::ReceiverToSender => (2.0, 0.0),
                    LinkDirection::Unknown => (0.0, 2.0),
                };

                // 解析 latency=XXms
                let mut latency = 0.0;
                if let Some(idx) = desc.find("latency=") {
                    let s = &desc[idx + "latency=".len()..];
                    if let Some(end_idx) = s.find("ms")
                        && let Ok(v) = s[..end_idx].trim().parse::<f64>()
                    {
                        latency = v;
                    }
                }
                let t1 = if latency > 0.0 {
                    t0 + latency
                } else {
                    t0 + 1.0
                };

                // 两段折线：src -> channel -> dst
                let mid_y = 1.0;
                let mid_t = (t0 + t1) / 2.0;
                lines.push(CanvasLine {
                    x1: t0,
                    y1: y_src,
                    x2: mid_t,
                    y2: mid_y,
                    color: Color::White,
                });
                lines.push(CanvasLine {
                    x1: mid_t,
                    y1: mid_y,
                    x2: t1,
                    y2: y_dst,
                    color: Color::White,
                });
            } else if desc.contains("DROP") {
                drop_points.push((t0, 1.0));
                annotations.push((
                    t0,
                    1.25,
                    format_link_annotation(desc, "DROP", direction),
                    Color::Red,
                ));
            } else if desc.contains("CORRUPT") {
                corrupt_points.push((t0, 1.0));
                annotations.push((
                    t0,
                    0.75,
                    format_link_annotation(desc, "CORRUPT", direction),
                    Color::Yellow,
                ));
            }
        }

        let y_min = -0.5;
        let y_max = 2.5;

        let annotations = annotations;
        let drop_points = drop_points;
        let corrupt_points = corrupt_points;

        let canvas = Canvas::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Link Space-Time Diagram"),
            )
            .x_bounds([t_min, t_max])
            .y_bounds([y_min, y_max])
            .paint(move |ctx| {
                // 水平线：Sender / Channel / Receiver
                ctx.draw(&CanvasLine {
                    x1: t_min,
                    y1: 0.0,
                    x2: t_max,
                    y2: 0.0,
                    color: Color::Cyan,
                });
                ctx.draw(&CanvasLine {
                    x1: t_min,
                    y1: 1.0,
                    x2: t_max,
                    y2: 1.0,
                    color: Color::Gray,
                });
                ctx.draw(&CanvasLine {
                    x1: t_min,
                    y1: 2.0,
                    x2: t_max,
                    y2: 2.0,
                    color: Color::Yellow,
                });

                // 标签（简单文本，不带样式）
                ctx.print(t_min, 0.0, "S");
                ctx.print(t_min, 1.0, "ch");
                ctx.print(t_min, 2.0, "R");

                // 画包的折线轨迹
                for line in &lines {
                    ctx.draw(line);
                }

                // 故障点
                if !drop_points.is_empty() {
                    ctx.draw(&Points {
                        coords: &drop_points,
                        color: Color::Red,
                    });
                }
                if !corrupt_points.is_empty() {
                    ctx.draw(&Points {
                        coords: &corrupt_points,
                        color: Color::Yellow,
                    });
                }

                for (x, y, label, color) in &annotations {
                    ctx.print(
                        *x,
                        *y,
                        Span::styled(label.clone(), Style::default().fg(*color)),
                    );
                }
            });

        f.render_widget(canvas, area);
    }

    fn render_link_events(&self, f: &mut Frame, area: Rect) {
        let events = &self.simulator.link_events;
        if events.is_empty() {
            let block = Paragraph::new("No link events yet")
                .block(Block::default().borders(Borders::ALL).title("Link Events"));
            f.render_widget(block, area);
            return;
        }

        let height = area.height.max(3) as usize;
        let visible = height - 2; // account for borders
        let total = events.len();
        let max_scroll = total.saturating_sub(visible);
        let scroll = self.link_scroll.min(max_scroll);
        let start = total.saturating_sub(visible + scroll);
        let end = total.saturating_sub(scroll);
        let start = start.max(0);
        let end = end.max(start);
        let slice = &events[start..end];

        let items: Vec<ListItem> = slice
            .iter()
            .map(|e| {
                let text = format!("[{:>5} ms] {}", e.time, e.description);
                let style = if e.description.contains("DROP") || e.description.contains("CORRUPT") {
                    Style::default().fg(Color::Red)
                } else if e.description.contains("DELIVERED") {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect();

        let list =
            List::new(items).block(Block::default().borders(Borders::ALL).title("Link Events"));

        f.render_widget(list, area);
    }
}

fn format_link_annotation(desc: &str, fallback: &str, direction: LinkDirection) -> String {
    const LIMIT: usize = 16;
    let keys: [&str; 2] = match direction {
        LinkDirection::SenderToReceiver => ["seq=", "ack="],
        LinkDirection::ReceiverToSender => ["ack=", "seq="],
        LinkDirection::Unknown => ["seq=", "ack="],
    };

    if let Some(field) = keys.into_iter().find_map(|key| extract_field(desc, key)) {
        let label = format!("{} {}", fallback, field);
        if label.len() > LIMIT {
            label[..LIMIT].to_string()
        } else {
            label
        }
    } else {
        fallback.to_string()
    }
}

fn extract_field(desc: &str, key: &str) -> Option<String> {
    let idx = desc.find(key)?;
    let rest = &desc[idx + key.len()..];
    let token = rest
        .split([' ', ')', '|'])
        .next()
        .unwrap_or("")
        .trim_matches(',');
    if token.is_empty() {
        None
    } else {
        Some(format!("{}{}", key, token))
    }
}

#[derive(Copy, Clone, Debug)]
enum LinkDirection {
    SenderToReceiver,
    ReceiverToSender,
    Unknown,
}

fn detect_direction(desc: &str) -> LinkDirection {
    if desc.contains("[Sender->Receiver]") {
        LinkDirection::SenderToReceiver
    } else if desc.contains("[Receiver->Sender]") {
        LinkDirection::ReceiverToSender
    } else {
        LinkDirection::Unknown
    }
}

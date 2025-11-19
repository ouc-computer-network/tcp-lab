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
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use tcp_lab_core::{Simulator, NodeId};

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
    
    pub fn get_lines(&self, max: usize) -> Vec<String> {
        let logs = self.logs.lock().unwrap();
        logs.iter().rev().take(max).rev().cloned().collect()
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
}

impl TuiApp {
    pub fn new(simulator: Simulator, log_buffer: MemoryLogBuffer) -> Self {
        Self {
            simulator,
            log_buffer,
            paused: true, // Start paused
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
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(f.area());

        // Left: Dashboard
        self.render_dashboard(f, chunks[0]);

        // Right: Logs
        self.render_logs(f, chunks[1]);
    }

    fn render_dashboard(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Status bar
                Constraint::Min(0),    // Main stats
            ])
            .split(area);

        let status_text = format!(
            "Time: {} ms | Status: {} | Events Pending: {} | (q)uit (space)pause/resume (s)tep",
            self.simulator.current_time(),
            if self.paused { "PAUSED" } else { "RUNNING" },
            self.simulator.remaining_events()
        );

        let status_block = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title("Control"));
        f.render_widget(status_block, chunks[0]);
        
        // Stats
        let stats_text = vec![
            Line::from("Simulation Stats:"),
            Line::from(format!("Sender: {:?}", NodeId::Sender)),
            Line::from(format!("Receiver: {:?}", NodeId::Receiver)),
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

    fn render_logs(&self, f: &mut Frame, area: Rect) {
        let height = area.height as usize;
        // Fetch slightly more lines than height to ensure coverage
        let logs = self.log_buffer.get_lines(height - 2);
        
        let items: Vec<ListItem> = logs
            .iter()
            .map(|log| ListItem::new(Line::from(log.as_str())))
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Logs"));
        
        f.render_widget(list, area);
    }
}

use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::{
    collectors::system::SystemCollector,
    models::metrics::{ProcessMetric, SystemSnapshot},
    ui,
};

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSort {
    Cpu,
    Memory,
    Name,
}

impl ProcessSort {
    fn next(self) -> Self {
        match self {
            Self::Cpu => Self::Memory,
            Self::Memory => Self::Name,
            Self::Name => Self::Cpu,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Memory => "MEMORY",
            Self::Name => "NAME",
        }
    }
}

pub struct App {
    collector: SystemCollector,
    pub snapshot: SystemSnapshot,
    pub sort_by: ProcessSort,
    pub descending: bool,
    pub should_quit: bool,
    last_refresh: Instant,
}

impl App {
    pub fn new() -> Result<Self, sysinfo::Error> {
        let mut collector = SystemCollector::new()?;
        collector.refresh();
        let snapshot = collector.snapshot();

        Ok(Self {
            collector,
            snapshot,
            sort_by: ProcessSort::Cpu,
            descending: true,
            should_quit: false,
            last_refresh: Instant::now(),
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| ui::render(frame, self))?;
            self.handle_events()?;
            self.refresh_if_due();
        }

        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if !event::poll(EVENT_POLL_INTERVAL)? {
            return Ok(());
        }

        if let Event::Key(key) = event::read()? {
            // Ignore key-release/repeat events so a held key doesn't cycle
            // through sort modes unexpectedly.
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                KeyCode::Char('s') => self.sort_by = self.sort_by.next(),
                KeyCode::Char('r') => self.descending = !self.descending,
                _ => {}
            }
        }

        Ok(())
    }

    fn refresh_if_due(&mut self) {
        if self.last_refresh.elapsed() < REFRESH_INTERVAL {
            return;
        }

        self.collector.refresh();
        self.snapshot = self.collector.snapshot();
        self.last_refresh = Instant::now();
    }

    pub fn sorted_processes(&self) -> Vec<ProcessMetric> {
        let mut processes = self.snapshot.processes.clone();

        processes.sort_by(|left, right| {
            let primary = match self.sort_by {
                ProcessSort::Cpu => left.cpu_percent.total_cmp(&right.cpu_percent),
                ProcessSort::Memory => left.memory_bytes.cmp(&right.memory_bytes),
                ProcessSort::Name => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
            };
            let primary = if self.descending {
                primary.reverse()
            } else {
                primary
            };

            // PID provides deterministic ordering when the selected values tie.
            primary.then_with(|| left.pid.cmp(&right.pid))
        });

        processes
    }
}

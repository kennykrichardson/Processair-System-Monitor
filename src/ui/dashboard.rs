use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, Wrap},
};

use crate::{
    app::App,
    models::metrics::{format_bytes, format_uptime},
};

pub fn render(frame: &mut Frame, app: &App) {
    let [header_area, metrics_area, processes_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(9),
        Constraint::Min(6),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_header(frame, header_area, app);
    render_metrics(frame, metrics_area, app);
    render_processes(frame, processes_area, app);
    render_footer(frame, footer_area, app);
}

fn render_header(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let title = Line::from(vec![
        Span::styled(
            " PROCESSAIR ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            "  {}  |  {}  |  uptime {}  |  {} processes ",
            app.snapshot.hostname,
            app.snapshot.operating_system,
            format_uptime(app.snapshot.uptime_seconds),
            app.snapshot.process_count
        )),
    ]);

    frame.render_widget(
        Paragraph::new(title).block(Block::default().borders(Borders::ALL)),
        area,
    );
}

fn render_metrics(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let [cpu_area, memory_area] =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(area);

    render_cpu(frame, cpu_area, app);
    render_memory(frame, memory_area, app);
}

fn render_cpu(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title(" CPU ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [gauge_area, detail_area] =
        Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).areas(inner);

    let cpu_percent = app.snapshot.cpu_percent.clamp(0.0, 100.0);
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .percent(cpu_percent.round() as u16)
        .label(format!("{cpu_percent:.1}% overall"));
    frame.render_widget(gauge, gauge_area);

    let core_summary = app
        .snapshot
        .cpu_cores
        .iter()
        .enumerate()
        .take(8)
        .map(|(index, usage)| format!("C{index}: {usage:>4.1}%"))
        .collect::<Vec<_>>()
        .join("   ");

    let hidden_cores = app.snapshot.cpu_cores.len().saturating_sub(8);
    let detail = if hidden_cores > 0 {
        format!("{core_summary}   +{hidden_cores} more")
    } else if core_summary.is_empty() {
        "Per-core data unavailable".to_owned()
    } else {
        core_summary
    };

    frame.render_widget(
        Paragraph::new(detail)
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: true }),
        detail_area,
    );
}

fn render_memory(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title(" MEMORY ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [ram_area, swap_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(2)]).areas(inner);

    let ram_percent = app.snapshot.memory_percent();
    let ram_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
        .percent(ram_percent.round() as u16)
        .label(format!(
            "RAM {:.1}%  {} / {}",
            ram_percent,
            format_bytes(app.snapshot.memory_used_bytes),
            format_bytes(app.snapshot.memory_total_bytes)
        ));
    frame.render_widget(ram_gauge, ram_area);

    let swap_percent = app.snapshot.swap_percent();
    let swap_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
        .percent(swap_percent.round() as u16)
        .label(format!(
            "SWAP {:.1}%  {} / {}",
            swap_percent,
            format_bytes(app.snapshot.swap_used_bytes),
            format_bytes(app.snapshot.swap_total_bytes)
        ));
    frame.render_widget(swap_gauge, swap_area);
}

fn render_processes(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let header = Row::new(vec!["PID", "PROCESS", "CPU %", "MEMORY", "PARENT"]).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    let visible_rows = area.height.saturating_sub(3) as usize;
    let rows = app
        .sorted_processes()
        .into_iter()
        .take(visible_rows)
        .map(|process| {
            let name = if process.name.is_empty() {
                "<unknown>".to_owned()
            } else {
                process.name
            };

            Row::new(vec![
                Cell::from(process.pid.to_string()),
                Cell::from(name),
                Cell::from(format!("{:.1}", process.cpu_percent)),
                Cell::from(format_bytes(process.memory_bytes)),
                Cell::from(
                    process
                        .parent_pid
                        .map_or_else(|| "-".to_owned(), |pid| pid.to_string()),
                ),
            ])
        });

    let title = format!(
        " PROCESSES  |  sort: {} {} ",
        app.sort_by.label(),
        if app.descending {
            "descending"
        } else {
            "ascending"
        }
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(9),
            Constraint::Min(20),
            Constraint::Length(9),
            Constraint::Length(12),
            Constraint::Length(9),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .column_spacing(1);

    frame.render_widget(table, area);
}

fn render_footer(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let footer = Line::from(vec![
        Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::raw(" quit   "),
        Span::styled(" s ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::raw(" cycle sort   "),
        Span::styled(" r ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::raw(" reverse order   "),
        Span::styled(
            format!(" {} ", app.snapshot.process_count),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" processes detected"),
    ]);

    frame.render_widget(Paragraph::new(footer), area);
}

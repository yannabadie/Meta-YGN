use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

use super::app::TuiApp;

/// Render the full dashboard to the terminal frame.
///
/// Layout (2x2 grid):
/// ```text
/// +---------------------+---------------------+
/// |  DAEMON STATUS      |  MEMORY STATS       |
/// +---------------------+---------------------+
/// |  FATIGUE            |  HELP               |
/// +---------------------+---------------------+
/// ```
pub fn render(frame: &mut Frame, app: &TuiApp) {
    let outer = frame.area();

    // Split into two rows
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer);

    // Split each row into two columns
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    render_daemon_status(frame, app, top[0]);
    render_memory_stats(frame, app, top[1]);
    render_fatigue(frame, app, bottom[0]);
    render_help(frame, bottom[1]);
}

// ---------------------------------------------------------------------------
// Top-left: Daemon Status
// ---------------------------------------------------------------------------

fn render_daemon_status(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" DAEMON STATUS ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut lines = Vec::new();

    if let Some(ref health) = app.health {
        let status = health
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let version = health
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let kernel = health
            .get("kernel")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let status_color = if status == "ok" {
            Color::Green
        } else {
            Color::Red
        };

        lines.push(Line::from(vec![
            Span::raw("  Status:  "),
            Span::styled(
                status.to_uppercase(),
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(format!("  Version: {version}")));
        lines.push(Line::from(format!("  Kernel:  {kernel}")));
        lines.push(Line::from(format!("  Port:    {}", app.daemon_port)));
    } else if let Some(ref err) = app.last_error {
        lines.push(Line::from(vec![
            Span::raw("  Status:  "),
            Span::styled(
                "OFFLINE",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(format!("  Port:    {}", app.daemon_port)));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!("  {err}"),
            Style::default().fg(Color::DarkGray),
        )]));
    } else {
        lines.push(Line::from("  Loading...").style(Style::default().fg(Color::DarkGray)));
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

// ---------------------------------------------------------------------------
// Top-right: Memory Stats
// ---------------------------------------------------------------------------

fn render_memory_stats(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" MEMORY STATS ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let mut lines = Vec::new();

    if let Some(ref stats) = app.memory_stats {
        let event_count = stats
            .get("event_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        lines.push(Line::from(vec![
            Span::raw("  Events:  "),
            Span::styled(
                event_count.to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // The daemon may return tier breakdowns in the future.
        // For now we display what's available.
        if let Some(hot) = stats.get("hot_count").and_then(|v| v.as_u64()) {
            lines.push(Line::from(format!("  Hot:     {hot}")));
        }
        if let Some(warm) = stats.get("warm_count").and_then(|v| v.as_u64()) {
            lines.push(Line::from(format!("  Warm:    {warm}")));
        }
        if let Some(cold) = stats.get("cold_count").and_then(|v| v.as_u64()) {
            lines.push(Line::from(format!("  Cold:    {cold}")));
        }
    } else {
        lines.push(Line::from("  Waiting for data...").style(Style::default().fg(Color::DarkGray)));
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

// ---------------------------------------------------------------------------
// Bottom-left: Fatigue
// ---------------------------------------------------------------------------

fn render_fatigue(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" FATIGUE ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    // We need to split the area for the text + gauge bar
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Reserve the last 3 lines for the gauge bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let text_area = chunks[0];
    let gauge_area = chunks[1];

    let mut lines = Vec::new();

    if let Some(ref fatigue) = app.fatigue {
        let score = fatigue
            .get("score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let high_friction = fatigue
            .get("high_friction")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let signals = fatigue
            .get("signals")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        let mode_str = if high_friction {
            "High-Friction"
        } else {
            "Normal"
        };

        let score_color = if score >= 0.7 {
            Color::Red
        } else if score > 0.4 {
            Color::Yellow
        } else {
            Color::Green
        };

        lines.push(Line::from(vec![
            Span::raw("  Score:   "),
            Span::styled(
                format!("{score:.2}"),
                Style::default()
                    .fg(score_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(format!("  Mode:    {mode_str}")));
        lines.push(Line::from(format!("  Signals: {signals}")));

        let text = Paragraph::new(lines);
        frame.render_widget(text, text_area);

        // Render the gauge bar
        let percent = (score * 100.0).round().min(100.0) as u16;
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(score_color).bg(Color::DarkGray))
            .percent(percent)
            .label(format!("{percent}%"));
        frame.render_widget(gauge, gauge_area);
    } else {
        lines.push(Line::from("  Score:   --").style(Style::default().fg(Color::DarkGray)));
        lines.push(Line::from("  Mode:    --").style(Style::default().fg(Color::DarkGray)));
        lines.push(
            Line::from("  (endpoint not available)")
                .style(Style::default().fg(Color::DarkGray)),
        );

        let text = Paragraph::new(lines);
        frame.render_widget(text, text_area);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::DarkGray).bg(Color::DarkGray))
            .percent(0)
            .label("--");
        frame.render_widget(gauge, gauge_area);
    }
}

// ---------------------------------------------------------------------------
// Bottom-right: Help
// ---------------------------------------------------------------------------

fn render_help(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" HELP ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  q", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" / "),
            Span::styled("Esc", Style::default().fg(Color::Cyan).bold()),
            Span::raw("    Quit"),
        ]),
        Line::from(vec![
            Span::styled("  r", Style::default().fg(Color::Cyan).bold()),
            Span::raw("            Force refresh"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+C", Style::default().fg(Color::Cyan).bold()),
            Span::raw("       Quit"),
        ]),
        Line::from(""),
        Line::from("  Dashboard refreshes every 500ms").style(Style::default().fg(Color::DarkGray)),
        Line::from("  Polling daemon HTTP API").style(Style::default().fg(Color::DarkGray)),
    ];

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

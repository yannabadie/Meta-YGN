pub mod app;
pub mod ui;

use std::io;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{self, Event};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use self::app::TuiApp;
use self::ui::render;

/// Run the interactive TUI dashboard.
///
/// This takes ownership of stdout, enters the alternate screen, and polls
/// the daemon HTTP API every 500 ms until the user presses `q` or `Esc`.
pub async fn run_tui(port: u16) -> Result<()> {
    // Setup terminal
    terminal::enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)
        .context("failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("failed to create terminal")?;

    let mut app = TuiApp::new(port);
    let tick_rate = Duration::from_millis(500);

    // Initial data fetch before first render
    app.tick().await;

    let result = run_event_loop(&mut terminal, &mut app, tick_rate).await;

    // Restore terminal — always attempt even if the loop errored
    terminal::disable_raw_mode().ok();
    crossterm::execute!(io::stdout(), LeaveAlternateScreen).ok();

    result
}

/// Inner event loop, factored out so cleanup always runs.
async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TuiApp,
    tick_rate: Duration,
) -> Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app))?;

        // Wait up to `tick_rate` for a keyboard event
        if event::poll(tick_rate)?
            && let Event::Key(key) = event::read()?
        {
            app.handle_key(key);
        }

        if app.should_quit {
            break;
        }

        app.tick().await;
    }

    Ok(())
}

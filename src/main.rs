mod app;
mod colors;
mod event;
mod slurm;
mod ui;
mod worker;

use std::io;
use std::time::Duration;

use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;
use event::{AppEvent, poll_event};

const POLL_INTERVAL: Duration = Duration::from_millis(100);

fn main() -> io::Result<()> {
    colors::init();

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = io::stdout().execute(LeaveAlternateScreen);
        original_hook(info);
    }));

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        app.poll_worker();

        match poll_event(POLL_INTERVAL) {
            AppEvent::Key(key) => app.on_key(key),
            AppEvent::Tick => app.tick(),
        }

        app.poll_worker();

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

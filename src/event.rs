use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub fn poll_event(timeout: Duration) -> AppEvent {
    if crossterm::event::poll(timeout).unwrap_or(false) {
        match event::read() {
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => AppEvent::Key(key),
            _ => AppEvent::Tick,
        }
    } else {
        AppEvent::Tick
    }
}

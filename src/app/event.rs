use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io;
use tokio::time::Duration;

use super::AppMode;

pub enum AppEvent {
    ChangeMode(AppMode),
    Exit,
}

pub fn handle_event(mode: &AppMode) -> io::Result<Option<AppEvent>> {
    match event::poll(Duration::from_millis(10)) {
        Ok(true) => match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => match mode {
                AppMode::Normal => Ok(handle_normal_mode(key_event.code)),
                AppMode::Input => Ok(handle_input_mode(key_event.code)),
            },
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

fn handle_normal_mode(key: KeyCode) -> Option<AppEvent> {
    match key {
        KeyCode::Enter => Some(AppEvent::ChangeMode(AppMode::Input)),
        KeyCode::Char('q') => Some(AppEvent::Exit),
        _ => None,
    }
}

fn handle_input_mode(key: KeyCode) -> Option<AppEvent> {
    match key {
        KeyCode::Esc => Some(AppEvent::ChangeMode(AppMode::Normal)),
        _ => None,
    }
}

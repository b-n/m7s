use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io;
use tokio::time::Duration;

use super::AppMode;

pub enum AppEvent {
    ChangeMode(AppMode),
    Exit,
    Submit,
    Load,
    CursorY(isize),
}

pub fn handle_event(mode: &AppMode) -> io::Result<Option<AppEvent>> {
    match event::poll(Duration::from_millis(10)) {
        Ok(true) => match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => match mode {
                AppMode::Normal => Ok(handle_normal_mode(key_event)),
                AppMode::Input => Ok(handle_input_mode(key_event)),
            },
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

fn handle_normal_mode(event: KeyEvent) -> Option<AppEvent> {
    let code = event.code;
    let modifiers = event.modifiers;
    match (code, modifiers) {
        (KeyCode::Enter, _) => Some(AppEvent::ChangeMode(AppMode::Input)),
        (KeyCode::Char('q'), _) => Some(AppEvent::Exit),
        (KeyCode::Char('l'), KeyModifiers::CONTROL) => Some(AppEvent::Load),
        (KeyCode::Up | KeyCode::Char('k'), _) => Some(AppEvent::CursorY(-1)),
        (KeyCode::Down | KeyCode::Char('j'), _) => Some(AppEvent::CursorY(1)),

        _ => None,
    }
}

fn handle_input_mode(event: KeyEvent) -> Option<AppEvent> {
    let code = event.code;
    match code {
        KeyCode::Esc => Some(AppEvent::ChangeMode(AppMode::Normal)),
        KeyCode::Enter => Some(AppEvent::Submit),
        _ => None,
    }
}
